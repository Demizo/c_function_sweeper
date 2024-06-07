use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, usize};

use clap::Parser;
use tree_sitter::{Node, Parser as TsParser};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Simple C function sweeper",
    long_about = "Search for unused or undeclared C functions"
)]
struct Args {
    /// Path or file to sweep
    #[arg(short, long)]
    path: String,

    /// Search folders recursively
    #[arg(short, long)]
    recursive: bool,
}

#[derive(Default)]
struct FunctionStats {
    declarations: Vec<(PathBuf, usize, usize)>,
    calls: Vec<(PathBuf, usize, usize)>,
}

fn main() {
    // Parse arguments
    let args = Args::parse();
    let path = Path::new(&args.path);
    let recursive = args.recursive;

    // Setup Tree-sitter parser
    let mut parser = TsParser::new();
    parser
        .set_language(&tree_sitter_c::language())
        .expect("Error loading C grammar");

    // Track function stats
    let mut function_stats: HashMap<String, FunctionStats> = HashMap::new();

    if path.is_dir() {
        // Traverse the directory and find C source and header files
        if let Ok(entries) = fs::read_dir(path) {
            if recursive {
                // Search recursively
                for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
                    let file_path = entry.path();
                    if is_source_or_header_file(&file_path) {
                        parse_file(file_path, &mut parser, &mut function_stats);
                    }
                }
            } else {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_path = entry.path();
                        if is_source_or_header_file(&file_path) {
                            parse_file(&file_path, &mut parser, &mut function_stats);
                        }
                    }
                }
            }
        } else {
            eprintln!("Could not read directory: {}", path.display());
        }
    } else if path.is_file() {
        if is_source_or_header_file(&path) {
            parse_file(&path, &mut parser, &mut function_stats);
        } else {
            eprintln!(
                "The specified file is not a C source or header file: {}",
                path.display()
            );
        }
    } else {
        eprintln!(
            "The specified path is neither a file nor a directory: {}",
            path.display()
        );
    }

    // Print the function stats
    for (function_name, stats) in function_stats {
        if function_name == "main" {
            continue;
        };
        if stats.calls.len() == 0 {
            println!("Unused Function '{}':", function_name);
            for (file, line, col) in stats.declarations.iter() {
                println!("-> {} {}:{}", file.display(), line, col);
            }
        }
        if stats.declarations.len() < 2 {
            println!("Undeclared Function '{}':", function_name);
            for (file, line, col) in stats.declarations.iter() {
                println!("-> {} {}:{}", file.display(), line, col);
            }
        }
    }
}

fn is_source_or_header_file(path: &Path) -> bool {
    return path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "c" || s == "h")
        .unwrap_or(false);
}

fn parse_file(
    path: &Path,
    parser: &mut TsParser,
    function_stats: &mut HashMap<String, FunctionStats>,
) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Some(tree) = parser.parse(&content, None) {
            // Find and update function declarations and calls
            find_function_stats(tree.root_node(), path, function_stats, content.as_bytes());
        } else {
            eprintln!("Could not parse file: {}", path.display());
        }
    } else {
        eprintln!("Could not read file: {}", path.display());
    }
}

fn find_function_stats(
    node: Node,
    path: &Path,
    function_stats: &mut HashMap<String, FunctionStats>,
    source: &[u8],
) {
    // Traverse the syntax tree to find function declarations and call nodes
    match node.kind() {
        "function_declarator" => {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                let function_name = declarator.utf8_text(source).unwrap();
                let stats = function_stats.entry(function_name.to_string()).or_default();
                stats.declarations.push((
                    path.to_path_buf(),
                    declarator.start_position().row,
                    declarator.start_position().column,
                ));
            }
        }
        "call_expression" => {
            if let Some(function_name_node) = node.child_by_field_name("function") {
                let function_name = function_name_node.utf8_text(source).unwrap();
                let stats = function_stats.entry(function_name.to_string()).or_default();
                stats.calls.push((
                    path.to_path_buf(),
                    function_name_node.start_position().row,
                    function_name_node.start_position().column,
                ));
            }
        }
        _ => {}
    }

    // Recursively search the child nodes
    for child in node.children(&mut node.walk()) {
        find_function_stats(child, path, function_stats, source);
    }
}
