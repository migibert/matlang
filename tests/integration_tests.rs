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
    
    assert!(result.is_ok(), "BJJ example system should parse successfully");
    let system = result.unwrap();
    
    assert_eq!(system.name, "bjj-basic");
    
    // Verify BJJ roles
    assert!(system.roles.contains("Top"));
    assert!(system.roles.contains("Bottom"));
    assert!(system.roles.contains("Neutral"));
    
    // Verify some key states
    assert!(system.states.contains_key("Standing"));
    assert!(system.states.contains_key("ClosedGuard"));
    assert!(system.states.contains_key("Mount"));
    
    // Verify sequences
    assert!(system.sequences.contains_key("BasicTakedownToGuard"));
    assert!(system.sequences.contains_key("GuardPassSequence"));
    assert!(system.sequences.contains_key("MountToSubmission"));
}

#[test]
fn test_muay_thai_example_system() {
    let result = parse_martial_system("examples/muay-thai-basic");
    
    assert!(result.is_ok(), "Muay Thai example system should parse successfully");
    let system = result.unwrap();
    
    assert_eq!(system.name, "muay-thai-basic");
    
    // Verify Muay Thai roles
    assert!(system.roles.contains("Orthodox"));
    assert!(system.roles.contains("Southpaw"));
    assert!(system.roles.contains("Dominant"));
    assert!(system.roles.contains("Controlled"));
    
    // Verify states
    assert!(system.states.contains_key("StrikingRange"));
    assert!(system.states.contains_key("ThaiClinch"));
    
    // Verify sequences
    assert!(system.sequences.contains_key("JabCrossClinch"));
    assert!(system.sequences.contains_key("LowKickCombo"));
    assert!(system.sequences.contains_key("ClinchKneeSequence"));
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
