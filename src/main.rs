mod ast;
mod lexer;
mod parser;
mod semantic;
mod graph;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    println!("mat - Martial Art Tool v0.1.0");
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "validate" => {
            if args.len() < 3 {
                eprintln!("Error: validate requires a path argument");
                print_usage();
                process::exit(1);
            }
            validate_command(&args[2]);
        }
        "graph" => {
            if args.len() < 3 {
                eprintln!("Error: graph requires a path argument");
                print_usage();
                process::exit(1);
            }
            graph_command(&args[2]);
        }
        "dot" => {
            if args.len() < 3 {
                eprintln!("Error: dot requires a path argument");
                print_usage();
                process::exit(1);
            }
            dot_command(&args[2]);
        }
        "stats" => {
            if args.len() < 3 {
                eprintln!("Error: stats requires a path argument");
                print_usage();
                process::exit(1);
            }
            stats_command(&args[2]);
        }
        path if Path::new(path).exists() => {
            // Backwards compatibility: treat as validate
            validate_command(path);
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("\nUsage:");
    println!("  mat validate <directory>     # Validate a martial system");
    println!("  mat graph <directory>        # Export graph as JSON");
    println!("  mat dot <directory>          # Export graph as DOT (Graphviz)");
    println!("  mat stats <directory>        # Show graph statistics");
}

fn validate_command(path: &str) {
    let system = load_and_validate_system(path);
    
    println!("\n✓ System '{}' is valid!", system.name);
    println!("\nSystem summary:");
    println!("  Roles: {}", system.roles.len());
    for role in &system.roles {
        println!("    - {}", role);
    }
    println!("  States: {}", system.states.len());
    for state_name in system.states.keys() {
        println!("    - {}", state_name);
    }
    println!("  Sequences: {}", system.sequences.len());
    for seq_name in system.sequences.keys() {
        println!("    - {}", seq_name);
    }
}

fn graph_command(path: &str) {
    let system = load_and_validate_system(path);
    let graph = graph::MartialGraph::from_system(&system);
    
    match graph.to_json() {
        Ok(json) => {
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("Error exporting to JSON: {}", e);
            process::exit(1);
        }
    }
}

fn dot_command(path: &str) {
    let system = load_and_validate_system(path);
    let graph = graph::MartialGraph::from_system(&system);
    
    println!("{}", graph.to_dot());
}

fn stats_command(path: &str) {
    let system = load_and_validate_system(path);
    let graph = graph::MartialGraph::from_system(&system);
    let stats = graph.statistics();
    
    println!("\nGraph Statistics for '{}':", system.name);
    println!("  Nodes: {}", stats.node_count);
    println!("  Edges: {}", stats.edge_count);
    println!("  Self-loops: {}", stats.self_loops);
    
    if !stats.source_nodes.is_empty() {
        println!("\n  Source nodes (no incoming edges):");
        for node in &stats.source_nodes {
            println!("    - {}", node.id());
        }
    }
    
    if !stats.sink_nodes.is_empty() {
        println!("\n  Sink nodes (no outgoing edges):");
        for node in &stats.sink_nodes {
            println!("    - {}", node.id());
        }
    }
    
    if !stats.isolated_nodes.is_empty() {
        println!("\n  Isolated nodes (no connections):");
        for node in &stats.isolated_nodes {
            println!("    - {}", node.id());
        }
    }
    
    // Check for unreachable nodes
    let unreachable = graph.find_unreachable_nodes();
    if !unreachable.is_empty() {
        println!("\n  ⚠ Unreachable nodes:");
        for node in &unreachable {
            println!("    - {}", node.id());
        }
    }
}

fn load_and_validate_system(path: &str) -> semantic::MartialSystem {
    let path_obj = Path::new(path);
    
    if !path_obj.is_dir() {
        eprintln!("Error: '{}' is not a directory", path);
        process::exit(1);
    }
    
    println!("\nValidating martial system: {}", path);
    
    // Get system name from directory
    let system_name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Find all .martial files
    let martial_files = match find_martial_files(path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error finding .martial files: {}", e);
            process::exit(1);
        }
    };
    
    if martial_files.is_empty() {
        eprintln!("Error: No .martial files found in directory");
        process::exit(1);
    }
    
    println!("Found {} .martial files:", martial_files.len());
    for file in &martial_files {
        println!("  - {}", file);
    }
    
    // Parse all files
    let mut validator = semantic::SemanticValidator::new();
    
    for file_path in &martial_files {
        println!("\nParsing {}...", file_path);
        
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path, e);
                process::exit(1);
            }
        };
        
        // Lex
        let mut lexer = lexer::Lexer::new(&content);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Lexer error in {}: {}", file_path, e);
                process::exit(1);
            }
        };
        
        // Parse
        let mut parser = parser::Parser::new(tokens);
        let martial_file = match parser.parse() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Parse error in {}: {}", file_path, e);
                process::exit(1);
            }
        };
        
        // Add to validator
        if let Err(e) = validator.add_file(martial_file) {
            eprintln!("Semantic error in {}: {}", file_path, e);
            process::exit(1);
        }
        
        println!("  ✓ Parsed successfully");
    }
    
    // Validate the complete system
    println!("\nValidating system semantics...");
    match validator.validate(system_name.clone()) {
        Ok(system) => system,
        Err(e) => {
            eprintln!("\nValidation error: {}", e);
            process::exit(1);
        }
    }
}

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
