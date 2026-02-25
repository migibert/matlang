use std::fs;
use std::path::Path;

// Import from the martial-lang library
// Note: We need to reference modules through the crate name
extern crate martial_lang;

/// Helper function to load and parse all .martial files from a directory
fn parse_martial_system(dir_path: &str) -> Result<martial_lang::semantic::MartialSystem, String> {
    let path = Path::new(dir_path);
    
    if !path.is_dir() {
        return Err(format!("'{}' is not a directory", dir_path));
    }
    
    // Get system name from directory
    let system_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("test_system")
        .to_string();
    
    // Find all .martial files
    let martial_files = find_martial_files(dir_path)
        .map_err(|e| format!("Error finding .martial files: {}", e))?;
    
    if martial_files.is_empty() {
        return Err("No .martial files found in directory".to_string());
    }
    
    // Parse all files
    let mut validator = martial_lang::semantic::SemanticValidator::new();
    
    for file_path in &martial_files {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Error reading {}: {}", file_path, e))?;
        
        // Lex
        let mut lexer = martial_lang::lexer::Lexer::new(&content);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexer error in {}: {}", file_path, e))?;
        
        // Parse
        let mut parser = martial_lang::parser::Parser::new(tokens);
        let martial_file = parser
            .parse()
            .map_err(|e| format!("Parse error in {}: {}", file_path, e))?;
        
        // Add to validator
        validator
            .add_file(martial_file)
            .map_err(|e| format!("Semantic error in {}: {}", file_path, e))?;
    }
    
    // Validate the complete system
    validator
        .validate(system_name)
        .map_err(|e| format!("Validation error: {}", e))
}

/// Helper function to find all .martial files in a directory
fn find_martial_files(dir_path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "martial" {
                    if let Some(path_str) = path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    files.sort();
    Ok(files)
}

#[test]
fn test_valid_boxing_system() {
    let result = parse_martial_system("tests/fixtures/valid_simple");
    
    assert!(result.is_ok(), "Boxing system should parse successfully");
    let system = result.unwrap();
    
    assert_eq!(system.name, "valid_simple");
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Orthodox"));
    assert!(system.roles.contains("Southpaw"));
    
    // LongRange, MidRange, InsideRange
    assert_eq!(system.states.len(), 3);
    assert!(system.states.contains_key("LongRange"));
    assert!(system.states.contains_key("MidRange"));
    assert!(system.states.contains_key("InsideRange"));
    
    // Classic combos: 1-2, 1-2-3, body-head
    assert_eq!(system.sequences.len(), 3);
    assert!(system.sequences.contains_key("JabCross"));
    assert!(system.sequences.contains_key("JabCrossHook"));
    assert!(system.sequences.contains_key("BodyToHeadCombo"));
    
    // 1-2-3 should be jab, cross, lead hook closing distance
    let combo = &system.sequences["JabCrossHook"];
    assert_eq!(combo.steps.len(), 3);
    assert_eq!(combo.steps[0].action_name, "Jab");
    assert_eq!(combo.steps[1].action_name, "Cross");
    assert_eq!(combo.steps[2].action_name, "LeadHook");
    assert_eq!(combo.steps[0].from.state, "LongRange");
    assert_eq!(combo.steps[2].to.state, "InsideRange");
}

#[test]
fn test_valid_complex_wrestling_system() {
    let result = parse_martial_system("tests/fixtures/valid_complex");
    
    assert!(result.is_ok(), "Complex wrestling system should parse successfully");
    let system = result.unwrap();
    
    assert_eq!(system.name, "valid_complex");
    
    // Check roles
    assert_eq!(system.roles.len(), 3);
    assert!(system.roles.contains("Offensive"));
    assert!(system.roles.contains("Defensive"));
    assert!(system.roles.contains("Neutral"));
    
    // Check states (8 positions including FrontHeadlock)
    assert_eq!(system.states.len(), 8);
    assert!(system.states.contains_key("NeutralStance"));
    assert!(system.states.contains_key("CollarTie"));
    assert!(system.states.contains_key("DoubleUnderhooks"));
    assert!(system.states.contains_key("SingleLeg"));
    assert!(system.states.contains_key("DoubleLeg"));
    assert!(system.states.contains_key("FrontHeadlock"));
    assert!(system.states.contains_key("BackControl"));
    assert!(system.states.contains_key("TopRide"));
    
    // Check sequences (6 chains)
    assert_eq!(system.sequences.len(), 6);
    assert!(system.sequences.contains_key("CollarTieSnapdown"));
    assert!(system.sequences.contains_key("FrontHeadlockSeries"));
    assert!(system.sequences.contains_key("SingleLegTakedown"));
    assert!(system.sequences.contains_key("DoubleLegBlast"));
    assert!(system.sequences.contains_key("DoubleUnderhookSeries"));
    assert!(system.sequences.contains_key("DuckUnderBackTake"));
    
    // Single leg: shoot, run the pipe, lift and dump
    let single_leg = &system.sequences["SingleLegTakedown"];
    assert_eq!(single_leg.steps.len(), 3);
    assert_eq!(single_leg.steps[0].action_name, "ShootIn");
    assert_eq!(single_leg.steps[1].action_name, "RunThePipe");
    assert_eq!(single_leg.steps[2].action_name, "LiftAndDump");
}

#[test]
fn test_invalid_undefined_role() {
    let result = parse_martial_system("tests/fixtures/invalid_undefined_role");
    
    assert!(result.is_err(), "System with undefined role should fail validation");
    let error = result.unwrap_err();
    assert!(
        error.contains("Uke") || error.contains("role") || error.contains("undefined"),
        "Error should mention undefined role 'Uke', got: {}",
        error
    );
}

#[test]
fn test_invalid_undefined_state() {
    let result = parse_martial_system("tests/fixtures/invalid_undefined_state");
    
    assert!(result.is_err(), "System with undefined state should fail validation");
    let error = result.unwrap_err();
    assert!(
        error.contains("FightingStance") || error.contains("ClosingDistance") || error.contains("State"),
        "Error should mention undefined TKD state, got: {}",
        error
    );
}

#[test]
fn test_multi_file_roles() {
    let result = parse_martial_system("tests/fixtures/multi_file_roles");
    
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "System with roles split across files should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    // Roles should be merged from both files
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Striker"));
    assert!(system.roles.contains("Grappler"));
    
    // States should reference merged roles
    assert_eq!(system.states.len(), 2);
    let clinch = &system.states["Clinch"];
    assert_eq!(clinch.allowed_roles.as_ref().unwrap().len(), 2);
}

#[test]
fn test_bjj_example_system() {
    let result = parse_martial_system("examples/bjj-basic");
    
    assert!(result.is_ok(), "BJJ example system should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "bjj-basic");
    
    // Verify BJJ roles
    assert_eq!(system.roles.len(), 3);
    assert!(system.roles.contains("Top"));
    assert!(system.roles.contains("Bottom"));
    assert!(system.roles.contains("Neutral"));
    
    // 14 positions: full BJJ positional hierarchy
    assert_eq!(system.states.len(), 14);
    assert!(system.states.contains_key("Standing"));
    assert!(system.states.contains_key("ClosedGuard"));
    assert!(system.states.contains_key("OpenGuard"));
    assert!(system.states.contains_key("DeLaRivaGuard"));
    assert!(system.states.contains_key("HalfGuard"));
    assert!(system.states.contains_key("Headquarters"));
    assert!(system.states.contains_key("SideControl"));
    assert!(system.states.contains_key("KesaGatame"));
    assert!(system.states.contains_key("KneeOnBelly"));
    assert!(system.states.contains_key("Mount"));
    assert!(system.states.contains_key("RearMount"));
    assert!(system.states.contains_key("TurtlePosition"));
    assert!(system.states.contains_key("ArmbarPosition"));
    assert!(system.states.contains_key("TrianglePosition"));
    
    // 9 technique sequences
    assert_eq!(system.sequences.len(), 9);
    assert!(system.sequences.contains_key("GuardPullToBerimbolo"));
    assert!(system.sequences.contains_key("ToreandoPass"));
    assert!(system.sequences.contains_key("SideControlToMount"));
    assert!(system.sequences.contains_key("MountToArmbar"));
    assert!(system.sequences.contains_key("TriangleFromGuard"));
    assert!(system.sequences.contains_key("HalfGuardSweep"));
    assert!(system.sequences.contains_key("TurtleToRearMount"));
    assert!(system.sequences.contains_key("MountEscape"));
    assert!(system.sequences.contains_key("GiftWrapToBack"));
    
    // Berimbolo: starts neutral (guard pull), ends top (rear mount)
    let berimbolo = &system.sequences["GuardPullToBerimbolo"];
    assert_eq!(berimbolo.steps.len(), 4);
    assert_eq!(berimbolo.steps[0].from.role, "Neutral");
    assert_eq!(berimbolo.steps[3].action_name, "BeriboloSweep");
    assert_eq!(berimbolo.steps[3].to.state, "RearMount");
    assert_eq!(berimbolo.steps[3].to.role, "Top");
    
    // Triangle from guard: bottom player attacks from closed guard
    let triangle = &system.sequences["TriangleFromGuard"];
    assert_eq!(triangle.steps.len(), 4);
    assert_eq!(triangle.steps[0].from.state, "ClosedGuard");
    assert_eq!(triangle.steps[0].from.role, "Bottom");
    assert_eq!(triangle.steps[2].to.state, "TrianglePosition");
}

#[test]
fn test_muay_thai_example_system() {
    let result = parse_martial_system("examples/muay-thai-basic");
    
    assert!(result.is_ok(), "Muay Thai example system should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "muay-thai-basic");
    
    // Verify Muay Thai roles
    assert_eq!(system.roles.len(), 4);
    assert!(system.roles.contains("Orthodox"));
    assert!(system.roles.contains("Southpaw"));
    assert!(system.roles.contains("Dominant"));
    assert!(system.roles.contains("Controlled"));
    
    // 4 states: striking range, kicking range, thai clinch, single collar tie
    assert_eq!(system.states.len(), 4);
    assert!(system.states.contains_key("StrikingRange"));
    assert!(system.states.contains_key("KickingRange"));
    assert!(system.states.contains_key("ThaiClinch"));
    assert!(system.states.contains_key("SingleCollarTie"));
    
    // 6 sequences
    assert_eq!(system.sequences.len(), 6);
    assert!(system.sequences.contains_key("JabCrossLowMiddle"));
    assert!(system.sequences.contains_key("JabCrossClinch"));
    assert!(system.sequences.contains_key("TeepToLowKick"));
    assert!(system.sequences.contains_key("ClinchKneeSequence"));
    assert!(system.sequences.contains_key("ClinchToElbow"));
    assert!(system.sequences.contains_key("LowKickCombo"));
    
    // Jab, cross, low kick, middle kick - the classic Muay Thai combo
    let combo = &system.sequences["JabCrossLowMiddle"];
    assert_eq!(combo.steps.len(), 4);
    assert_eq!(combo.steps[0].action_name, "Jab");
    assert_eq!(combo.steps[2].action_name, "RearLegLowKick");
    assert_eq!(combo.steps[2].to.state, "KickingRange");
    assert_eq!(combo.steps[3].action_name, "LeadLegMiddleKick");
}

#[test]
fn test_graph_generation() {
    let result = parse_martial_system("tests/fixtures/valid_simple");
    assert!(result.is_ok());
    let system = result.unwrap();
    
    // Test graph generation
    let graph = martial_lang::graph::MartialGraph::from_system(&system);
    
    // Verify graph has nodes
    let stats = graph.statistics();
    assert!(stats.node_count > 0, "Graph should have nodes");
    assert!(stats.edge_count > 0, "Graph should have edges");
    
    // Test DOT export
    let dot = graph.to_dot();
    assert!(dot.contains("digraph"), "DOT output should contain digraph declaration");
    assert!(dot.contains("->"), "DOT output should contain edges");
    
    // Test JSON export
    let json_result = graph.to_json();
    assert!(json_result.is_ok(), "JSON export should succeed");
    let json = json_result.unwrap();
    assert!(json.contains("\"nodes\""), "JSON should contain nodes");
    assert!(json.contains("\"edges\""), "JSON should contain edges");
}

#[test]
fn test_graph_statistics() {
    let result = parse_martial_system("tests/fixtures/valid_complex");
    assert!(result.is_ok());
    let system = result.unwrap();
    
    let graph = martial_lang::graph::MartialGraph::from_system(&system);
    let stats = graph.statistics();
    
    // Wrestling system should have source nodes (starting positions)
    assert!(!stats.source_nodes.is_empty(), "Wrestling system should have source nodes");
    
    // Check for unreachable nodes
    let unreachable = graph.find_unreachable_nodes();
    // This is informational - some systems may have unreachable nodes by design
    println!("Unreachable nodes in valid_complex: {}", unreachable.len());
}

#[test]
fn test_sequence_connectivity() {
    let result = parse_martial_system("tests/fixtures/valid_complex");
    assert!(result.is_ok());
    let system = result.unwrap();
    
    // Verify CollarTieSnapdown chain: tie up then snap to front headlock
    let snapdown = &system.sequences["CollarTieSnapdown"];
    assert_eq!(snapdown.steps.len(), 2);
    
    let step1 = &snapdown.steps[0];
    let step2 = &snapdown.steps[1];
    
    // First step: InitialContact from neutral stance into collar tie
    assert_eq!(step1.action_name, "InitialContact");
    assert_eq!(step1.from.state, "NeutralStance");
    assert_eq!(step1.to.state, "CollarTie");
    
    // Second step: SnapDown from collar tie to front headlock
    assert_eq!(step2.action_name, "SnapDown");
    assert_eq!(step2.from.state, "CollarTie");
    assert_eq!(step2.to.state, "FrontHeadlock");
}

#[test]
fn test_valid_kata_system() {
    let result = parse_martial_system("tests/fixtures/valid_kata");
    
    assert!(result.is_ok(), "Karate kata system should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    // Hidari (left) and Migi (right) forward sides
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Hidari"));
    assert!(system.roles.contains("Migi"));
    
    // Stances: Yoi (ready), ZenkutsuDachi (front), KokutsuDachi (back)
    assert_eq!(system.states.len(), 3);
    assert!(system.states.contains_key("Yoi"));
    assert!(system.states.contains_key("ZenkutsuDachi"));
    assert!(system.states.contains_key("KokutsuDachi"));
    
    // Three lines of Heian Shodan
    assert_eq!(system.sequences.len(), 3);
    assert!(system.sequences.contains_key("HeianShodanFirstLine"));
    assert!(system.sequences.contains_key("HeianShodanSecondLine"));
    assert!(system.sequences.contains_key("HeianShodanThirdLine"));
    
    // First line: GedanBarai from Yoi, then ChudanOiZuki
    let first_line = &system.sequences["HeianShodanFirstLine"];
    assert_eq!(first_line.steps.len(), 2);
    assert_eq!(first_line.steps[0].action_name, "GedanBarai");
    assert_eq!(first_line.steps[0].from.state, "Yoi");
    assert_eq!(first_line.steps[1].action_name, "ChudanOiZuki");
    
    // Third line: gedan barai + three stepping age-uke (4 techniques)
    let third_line = &system.sequences["HeianShodanThirdLine"];
    assert_eq!(third_line.steps.len(), 4);
    assert_eq!(third_line.steps[3].action_name, "AgeUkeKiai");
}

#[test]
fn test_valid_poomsae_system() {
    let result = parse_martial_system("tests/fixtures/valid_poomsae");
    
    assert!(result.is_ok(), "Taekwondo poomsae should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    // Wen (left) and Orun (right) leading sides
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Wen"));
    assert!(system.roles.contains("Orun"));
    
    // Stances: Naranhi (ready), ApKubi (front), ApSeogi (walking)
    assert_eq!(system.states.len(), 3);
    assert!(system.states.contains_key("Naranhi"));
    assert!(system.states.contains_key("ApKubi"));
    assert!(system.states.contains_key("ApSeogi"));
    
    // Three lines of Taegeuk Il Jang
    assert_eq!(system.sequences.len(), 3);
    assert!(system.sequences.contains_key("TaegeukIlJangFirstLine"));
    assert!(system.sequences.contains_key("TaegeukIlJangSecondLine"));
    assert!(system.sequences.contains_key("TaegeukIlJangThirdLine"));
    
    // First line starts from Naranhi (ready stance), arae makgi into ap kubi
    let first_line = &system.sequences["TaegeukIlJangFirstLine"];
    assert_eq!(first_line.steps.len(), 2);
    assert_eq!(first_line.steps[0].action_name, "AraeMakgi");
    assert_eq!(first_line.steps[0].from.state, "Naranhi");
    assert_eq!(first_line.steps[0].to.state, "ApKubi");
    
    // Second line: momtong bandae jireugi step transition
    let second_line = &system.sequences["TaegeukIlJangSecondLine"];
    assert_eq!(second_line.steps[1].action_name, "MomtongBandaeJireugi");
}

#[test]
fn test_valid_bjj_system() {
    let result = parse_martial_system("tests/fixtures/valid_bjj");
    
    assert!(result.is_ok(), "BJJ system should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    // Top, Bottom, Neutral
    assert_eq!(system.roles.len(), 3);
    assert!(system.roles.contains("Top"));
    assert!(system.roles.contains("Bottom"));
    assert!(system.roles.contains("Neutral"));
    
    // 11 positions: the full BJJ positional hierarchy
    assert_eq!(system.states.len(), 11);
    assert!(system.states.contains_key("Standing"));
    assert!(system.states.contains_key("ClosedGuard"));
    assert!(system.states.contains_key("DeLaRivaGuard"));
    assert!(system.states.contains_key("Headquarters"));
    assert!(system.states.contains_key("KesaGatame"));
    assert!(system.states.contains_key("RearMount"));
    assert!(system.states.contains_key("TurtlePosition"));
    
    // 5 real technique sequences
    assert_eq!(system.sequences.len(), 5);
    assert!(system.sequences.contains_key("GuardPullToBerimbolo"));
    assert!(system.sequences.contains_key("ToreandoPass"));
    assert!(system.sequences.contains_key("GiftWrapToBack"));
    assert!(system.sequences.contains_key("HalfGuardRecovery"));
    assert!(system.sequences.contains_key("TurtleToRearMount"));
    
    // Berimbolo: starts bottom (guard pull), sweeps to top (rear mount)
    let berimbolo = &system.sequences["GuardPullToBerimbolo"];
    assert_eq!(berimbolo.steps.len(), 4);
    assert_eq!(berimbolo.steps[0].from.role, "Neutral");
    assert_eq!(berimbolo.steps[3].action_name, "BeriboloSweep");
    assert_eq!(berimbolo.steps[3].to.state, "RearMount");
    assert_eq!(berimbolo.steps[3].to.role, "Top");
    
    // Toreando pass: classic guard break to headquarters to smash pass
    let toreando = &system.sequences["ToreandoPass"];
    assert_eq!(toreando.steps.len(), 3);
    assert_eq!(toreando.steps[0].action_name, "BreakClosedGuard");
    assert_eq!(toreando.steps[1].to.state, "Headquarters");
    assert_eq!(toreando.steps[2].action_name, "ToreandoSmash");
    assert_eq!(toreando.steps[2].to.state, "SideControl");
}

#[test]
fn test_karate_heian_example() {
    let result = parse_martial_system("examples/karate-heian");
    
    assert!(result.is_ok(), "Karate Heian example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "karate-heian");
    
    // Hidari (left) and Migi (right)
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Hidari"));
    assert!(system.roles.contains("Migi"));
    
    // 4 stances: Yoi, ZenkutsuDachi, KokutsuDachi, KibaDachi
    assert_eq!(system.states.len(), 4);
    assert!(system.states.contains_key("Yoi"));
    assert!(system.states.contains_key("ZenkutsuDachi"));
    assert!(system.states.contains_key("KokutsuDachi"));
    assert!(system.states.contains_key("KibaDachi"));
    
    // Heian Shodan (3 lines) + Heian Nidan (2 lines) = 5 sequences
    assert_eq!(system.sequences.len(), 5);
    assert!(system.sequences.contains_key("HeianShodanFirstLine"));
    assert!(system.sequences.contains_key("HeianShodanSecondLine"));
    assert!(system.sequences.contains_key("HeianShodanThirdLine"));
    assert!(system.sequences.contains_key("HeianNidanFirstLine"));
    assert!(system.sequences.contains_key("HeianNidanSecondLine"));
    
    // Heian Nidan first line: starts Yoi, enters kokutsu-dachi
    let nidan_first = &system.sequences["HeianNidanFirstLine"];
    assert_eq!(nidan_first.steps.len(), 4);
    assert_eq!(nidan_first.steps[0].action_name, "JodanUchiUke");
    assert_eq!(nidan_first.steps[0].from.state, "Yoi");
    assert_eq!(nidan_first.steps[0].to.state, "KokutsuDachi");
    // Front kick transitions to zenkutsu dachi
    assert_eq!(nidan_first.steps[3].action_name, "MaeGeriKeage");
    assert_eq!(nidan_first.steps[3].to.state, "ZenkutsuDachi");
}

#[test]
fn test_aikido_kata_example() {
    let result = parse_martial_system("examples/aikido-kata");
    
    assert!(result.is_ok(), "Aikido kata example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "aikido-kata");
    
    // Tori and Uke
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Tori"));
    assert!(system.roles.contains("Uke"));
    
    // 8 states
    assert_eq!(system.states.len(), 8);
    assert!(system.states.contains_key("AiHanmi"));
    assert!(system.states.contains_key("GyakuHanmi"));
    assert!(system.states.contains_key("KatateDori"));
    assert!(system.states.contains_key("ShomenUchi"));
    assert!(system.states.contains_key("Irimi"));
    assert!(system.states.contains_key("Tenkan"));
    assert!(system.states.contains_key("Kuzushi"));
    assert!(system.states.contains_key("Zanshin"));
    
    // 4 technique sequences
    assert_eq!(system.sequences.len(), 4);
    assert!(system.sequences.contains_key("IkkyoFromKatateDori"));
    assert!(system.sequences.contains_key("Iriminage"));
    assert!(system.sequences.contains_key("ShihoNage"));
    assert!(system.sequences.contains_key("KoteGaeshi"));
    
    // Ikkyo: starts from gyaku hanmi (reverse stance), ends in zanshin
    let ikkyo = &system.sequences["IkkyoFromKatateDori"];
    assert_eq!(ikkyo.steps.len(), 4);
    assert_eq!(ikkyo.steps[0].from.state, "GyakuHanmi");
    assert_eq!(ikkyo.steps[3].to.state, "Zanshin");
    
    // All techniques end in Zanshin (maintained awareness)
    for (name, seq) in &system.sequences {
        let last_step = seq.steps.last().unwrap();
        assert_eq!(last_step.to.state, "Zanshin",
            "Sequence '{}' should end in Zanshin", name);
    }
}

#[test]
fn test_jujitsu_kata_example() {
    let result = parse_martial_system("examples/jujitsu-kata");
    
    assert!(result.is_ok(), "Traditional Jujitsu kata example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "jujitsu-kata");
    
    // Tori and Uke
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Tori"));
    assert!(system.roles.contains("Uke"));
    
    // 9 states
    assert_eq!(system.states.len(), 9);
    assert!(system.states.contains_key("Standing"));
    assert!(system.states.contains_key("KatateDori"));
    assert!(system.states.contains_key("MunadoriGrab"));
    assert!(system.states.contains_key("Clinch"));
    assert!(system.states.contains_key("HipLoaded"));
    assert!(system.states.contains_key("Kuzushi"));
    assert!(system.states.contains_key("GroundPin"));
    assert!(system.states.contains_key("JointLock"));
    assert!(system.states.contains_key("Zanshin"));
    
    // 4 technique sequences
    assert_eq!(system.sequences.len(), 4);
    assert!(system.sequences.contains_key("KoteGaeshi"));
    assert!(system.sequences.contains_key("OGoshiSequence"));
    assert!(system.sequences.contains_key("UdeGarami"));
    assert!(system.sequences.contains_key("SeoiNageToPin"));
    
    // O-goshi: hip throw with pin, goes through HipLoaded
    let ogoshi = &system.sequences["OGoshiSequence"];
    assert_eq!(ogoshi.steps.len(), 4);
    assert_eq!(ogoshi.steps[0].from.state, "Standing");
    assert_eq!(ogoshi.steps[1].to.state, "HipLoaded");
    assert_eq!(ogoshi.steps[2].to.state, "GroundPin");
    assert_eq!(ogoshi.steps[3].to.state, "Zanshin");
    
    // Ude-garami: ends in JointLock (not Zanshin - active lock)
    let ude_garami = &system.sequences["UdeGarami"];
    assert_eq!(ude_garami.steps.last().unwrap().to.state, "JointLock");
}

#[test]
fn test_taekwondo_poomsae_example() {
    let result = parse_martial_system("examples/taekwondo-poomsae");
    
    assert!(result.is_ok(), "Taekwondo poomsae example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "taekwondo-poomsae");
    
    // Wen (left) and Orun (right)
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Wen"));
    assert!(system.roles.contains("Orun"));
    
    // 5 stances: Naranhi, ApKubi, ApSeogi, DwitKubi, BeomSeogi
    assert_eq!(system.states.len(), 5);
    assert!(system.states.contains_key("Naranhi"));
    assert!(system.states.contains_key("ApKubi"));
    assert!(system.states.contains_key("ApSeogi"));
    assert!(system.states.contains_key("DwitKubi"));
    assert!(system.states.contains_key("BeomSeogi"));
    
    // Il Jang (3 lines) + Ee Jang (3 lines) = 6 sequences
    assert_eq!(system.sequences.len(), 6);
    assert!(system.sequences.contains_key("TaegeukIlJangFirstLine"));
    assert!(system.sequences.contains_key("TaegeukIlJangSecondLine"));
    assert!(system.sequences.contains_key("TaegeukIlJangThirdLine"));
    assert!(system.sequences.contains_key("TaegeukEeJangFirstLine"));
    assert!(system.sequences.contains_key("TaegeukEeJangSecondLine"));
    assert!(system.sequences.contains_key("TaegeukEeJangThirdLine"));
    
    // Ee Jang first line: includes ap chagi (front kick)
    let ee_jang_first = &system.sequences["TaegeukEeJangFirstLine"];
    assert_eq!(ee_jang_first.steps.len(), 3);
    assert_eq!(ee_jang_first.steps[1].action_name, "ApChagi");
}

#[test]
fn test_judo_newaza_example() {
    let result = parse_martial_system("examples/judo-newaza");
    
    assert!(result.is_ok(), "Judo example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "judo-newaza");
    
    // Tori and Uke
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Tori"));
    assert!(system.roles.contains("Uke"));
    
    // 7 states
    assert_eq!(system.states.len(), 7);
    assert!(system.states.contains_key("ShizenTai"));
    assert!(system.states.contains_key("KumiKata"));
    assert!(system.states.contains_key("Kuzushi"));
    assert!(system.states.contains_key("Tsukuri"));
    assert!(system.states.contains_key("NeWaza"));
    assert!(system.states.contains_key("Osaekomi"));
    assert!(system.states.contains_key("Turtle"));
    
    // 4 sequences
    assert_eq!(system.sequences.len(), 4);
    assert!(system.sequences.contains_key("UchiMataToOuchiGari"));
    assert!(system.sequences.contains_key("OsotoToKosoto"));
    assert!(system.sequences.contains_key("SeoiNageToOsaekomi"));
    assert!(system.sequences.contains_key("TurtleTurnover"));
    
    // Uchi-mata to ouchi-gari chain: failed throw chains to second throw
    let uchi_mata = &system.sequences["UchiMataToOuchiGari"];
    assert_eq!(uchi_mata.steps.len(), 5);
    assert_eq!(uchi_mata.steps[0].action_name, "EstablishGrip");
    assert_eq!(uchi_mata.steps[0].from.state, "ShizenTai");
    // Uchi-mata fails, returns to kumi kata
    assert_eq!(uchi_mata.steps[3].action_name, "UchiMataFails");
    assert_eq!(uchi_mata.steps[3].to.state, "KumiKata");
    // Ouchi-gari finishes to ne-waza
    assert_eq!(uchi_mata.steps[4].action_name, "OuchiGariReap");
    assert_eq!(uchi_mata.steps[4].to.state, "NeWaza");
    
    // Seoi-nage to osaekomi: throw to immediate pin
    let seoi = &system.sequences["SeoiNageToOsaekomi"];
    assert_eq!(seoi.steps.len(), 5);
    assert_eq!(seoi.steps[4].action_name, "KesaGatamePin");
    assert_eq!(seoi.steps[4].to.state, "Osaekomi");
}

#[test]
fn test_boxing_combos_example() {
    let result = parse_martial_system("examples/boxing-combos");
    
    assert!(result.is_ok(), "Boxing combos example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "boxing-combos");
    
    // Orthodox and Southpaw
    assert_eq!(system.roles.len(), 2);
    assert!(system.roles.contains("Orthodox"));
    assert!(system.roles.contains("Southpaw"));
    
    // 4 states: LongRange, MidRange, InsideRange, Clinch
    assert_eq!(system.states.len(), 4);
    assert!(system.states.contains_key("LongRange"));
    assert!(system.states.contains_key("MidRange"));
    assert!(system.states.contains_key("InsideRange"));
    assert!(system.states.contains_key("Clinch"));
    
    // 5 combinations
    assert_eq!(system.sequences.len(), 5);
    assert!(system.sequences.contains_key("JabCross"));
    assert!(system.sequences.contains_key("JabCrossHookCross"));
    assert!(system.sequences.contains_key("BodyAttack"));
    assert!(system.sequences.contains_key("SouthpawCounter"));
    assert!(system.sequences.contains_key("InsideFightingToClinch"));
    
    // Body attack: jab, cross, hook to body, uppercut
    let body_attack = &system.sequences["BodyAttack"];
    assert_eq!(body_attack.steps.len(), 4);
    assert_eq!(body_attack.steps[2].action_name, "LeadHookBody");
    assert_eq!(body_attack.steps[3].action_name, "RearUppercut");
    
    // Southpaw counter uses Southpaw role throughout
    let southpaw = &system.sequences["SouthpawCounter"];
    for step in &southpaw.steps {
        assert_eq!(step.from.role, "Southpaw");
    }
    
    // Inside fighting reaches clinch
    let clinch_seq = &system.sequences["InsideFightingToClinch"];
    assert_eq!(clinch_seq.steps.last().unwrap().to.state, "Clinch");
}

#[test]
fn test_wrestling_folkstyle_example() {
    let result = parse_martial_system("examples/wrestling-folkstyle");
    
    assert!(result.is_ok(), "Wrestling folkstyle example should parse successfully: {:?}", result);
    let system = result.unwrap();
    
    assert_eq!(system.name, "wrestling-folkstyle");
    
    // Offensive, Defensive, Neutral
    assert_eq!(system.roles.len(), 3);
    assert!(system.roles.contains("Offensive"));
    assert!(system.roles.contains("Defensive"));
    assert!(system.roles.contains("Neutral"));
    
    // 11 positions
    assert_eq!(system.states.len(), 11);
    assert!(system.states.contains_key("NeutralStance"));
    assert!(system.states.contains_key("CollarTie"));
    assert!(system.states.contains_key("Underhook"));
    assert!(system.states.contains_key("DoubleUnderhooks"));
    assert!(system.states.contains_key("SingleLeg"));
    assert!(system.states.contains_key("DoubleLeg"));
    assert!(system.states.contains_key("FrontHeadlock"));
    assert!(system.states.contains_key("BackControl"));
    assert!(system.states.contains_key("TopRide"));
    assert!(system.states.contains_key("RefereePosition"));
    assert!(system.states.contains_key("LegsIn"));
    
    // 6 sequences
    assert_eq!(system.sequences.len(), 6);
    assert!(system.sequences.contains_key("SnapdownSeries"));
    assert!(system.sequences.contains_key("SingleLegRunPipe"));
    assert!(system.sequences.contains_key("DoubleLegBlast"));
    assert!(system.sequences.contains_key("BodyLockTakedown"));
    assert!(system.sequences.contains_key("StandUp"));
    assert!(system.sequences.contains_key("LegRideSeries"));
    
    // Stand up: defensive wrestler escapes from referee position to neutral
    let standup = &system.sequences["StandUp"];
    assert_eq!(standup.steps.len(), 3);
    assert_eq!(standup.steps[0].from.state, "RefereePosition");
    assert_eq!(standup.steps[0].from.role, "Defensive");
    assert_eq!(standup.steps[2].to.state, "NeutralStance");
    assert_eq!(standup.steps[2].to.role, "Neutral");
    
    // Leg ride series: top ride to legs in, then turn
    let leg_ride = &system.sequences["LegRideSeries"];
    assert_eq!(leg_ride.steps.len(), 3);
    assert_eq!(leg_ride.steps[1].to.state, "LegsIn");
}

#[test]
fn test_empty_directory() {
    // Create a temporary empty directory
    let temp_dir = "tests/fixtures/empty_test_dir";
    if !Path::new(temp_dir).exists() {
        fs::create_dir_all(temp_dir).unwrap();
    }
    
    let result = parse_martial_system(temp_dir);
    assert!(result.is_err(), "Empty directory should fail");
    assert!(result.unwrap_err().contains("No .martial files found"));
    
    // Clean up
    fs::remove_dir(temp_dir).ok();
}
