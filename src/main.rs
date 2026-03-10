//! Forge CLI entrypoint.

mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod semantic;
mod token;
mod units;

use ast::Program;
use std::path::Path;

use error::{ForgeError, ForgeResult};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;

fn parse_and_validate_source(source: &str) -> ForgeResult<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let analyzer = SemanticAnalyzer::new();
    analyzer.validate(&program)?;

    Ok(program)
}

fn run_source(source: &str) -> ForgeResult<Vec<String>> {
    let program = parse_and_validate_source(source)?;

    let mut interpreter = Interpreter::new();
    interpreter.evaluate(&program)
}

fn check_source(source: &str) -> ForgeResult<()> {
    let _ = parse_and_validate_source(source)?;
    Ok(())
}

fn run_file(path: &Path) -> ForgeResult<Vec<String>> {
    let source = std::fs::read_to_string(path).map_err(|error| {
        ForgeError::new(format!(
            "Failed to read script '{}': {error}",
            path.display()
        ))
    })?;
    run_source(&source)
}

fn check_file(path: &Path) -> ForgeResult<()> {
    let source = std::fs::read_to_string(path).map_err(|error| {
        ForgeError::new(format!(
            "Failed to read script '{}': {error}",
            path.display()
        ))
    })?;
    check_source(&source)
}

fn usage(program: &str) -> String {
    format!(
        "Usage:\n  {program} run <file>\n  {program} check <file>\n  {program} version"
    )
}

fn split_message(message: &str) -> (&str, Vec<&str>) {
    let mut lines = message.lines();
    let headline = lines.next().unwrap_or("Unknown Forge error.");
    let details = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    (headline, details)
}

fn format_cli_error(path: &Path, error: &ForgeError) -> String {
    let (headline, details) = split_message(&error.message);

    let mut rendered = format!("error: {headline}");
    match (error.line, error.column) {
        (Some(line), Some(column)) => {
            rendered.push_str(&format!("\n  --> {}:{}:{}", path.display(), line, column))
        }
        _ => rendered.push_str(&format!("\n  --> {}", path.display())),
    }

    if !details.is_empty() {
        rendered.push_str("\n   |");
        for detail in details {
            rendered.push_str(&format!("\n   = {detail}"));
        }
    }

    rendered
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "forge".to_string());

    let Some(command) = args.next() else {
        eprintln!("{}", usage(&program));
        std::process::exit(2);
    };

    match command.as_str() {
        "help" | "-h" | "--help" => {
            if args.next().is_some() {
                eprintln!("The 'help' command does not take arguments.\n\n{}", usage(&program));
                std::process::exit(2);
            }
            println!("{}", usage(&program));
        }
        "run" => {
            let Some(path) = args.next() else {
                eprintln!(
                    "Missing file path for 'run' command.\n\n{}",
                    usage(&program)
                );
                std::process::exit(2);
            };

            if args.next().is_some() {
                eprintln!("Too many arguments for 'run' command.\n\n{}", usage(&program));
                std::process::exit(2);
            }

            match run_file(Path::new(&path)) {
                Ok(output) => {
                    for line in output {
                        println!("{line}");
                    }
                }
                Err(error) => {
                    eprintln!("{}", format_cli_error(Path::new(&path), &error));
                    std::process::exit(1);
                }
            }
        }
        "check" => {
            let Some(path) = args.next() else {
                eprintln!(
                    "Missing file path for 'check' command.\n\n{}",
                    usage(&program)
                );
                std::process::exit(2);
            };

            if args.next().is_some() {
                eprintln!("Too many arguments for 'check' command.\n\n{}", usage(&program));
                std::process::exit(2);
            }

            match check_file(Path::new(&path)) {
                Ok(()) => {
                    println!("Check passed: {}", path);
                }
                Err(error) => {
                    eprintln!("{}", format_cli_error(Path::new(&path), &error));
                    std::process::exit(1);
                }
            }
        }
        "version" => {
            if args.next().is_some() {
                eprintln!("The 'version' command does not take arguments.\n\n{}", usage(&program));
                std::process::exit(2);
            }
            println!("forge {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Unknown command '{}'.\n\n{}", command, usage(&program));
            std::process::exit(2);
        }
    }
}
