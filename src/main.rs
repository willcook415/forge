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
use semantic::{AnalysisReport, SemanticAnalyzer};
use units::UnitRegistry;

const DESCRIPTION: &str =
    "Forge is a unit-safe engineering worksheet language with dimensional analysis.";

fn parse_source(source: &str) -> ForgeResult<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn parse_and_analyze_source(source: &str) -> ForgeResult<(Program, AnalysisReport)> {
    let program = parse_source(source)?;
    let analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&program)?;

    Ok((program, report))
}

fn parse_and_validate_source(source: &str) -> ForgeResult<Program> {
    parse_and_analyze_source(source).map(|(program, _)| program)
}

fn run_source(source: &str) -> ForgeResult<Vec<String>> {
    let program = parse_and_validate_source(source)?;

    let mut interpreter = Interpreter::new();
    interpreter.evaluate(&program)
}

fn check_source(source: &str) -> ForgeResult<()> {
    let program = parse_source(source)?;
    let analyzer = SemanticAnalyzer::new();
    analyzer.validate(&program)?;
    Ok(())
}

fn read_script(path: &Path) -> ForgeResult<String> {
    std::fs::read_to_string(path).map_err(|error| {
        ForgeError::new(format!(
            "Failed to read script '{}': {error}",
            path.display()
        ))
    })
}

fn run_file(path: &Path) -> ForgeResult<Vec<String>> {
    let source = read_script(path)?;
    run_source(&source)
}

fn check_file(path: &Path) -> ForgeResult<()> {
    let source = read_script(path)?;
    check_source(&source)
}

fn explain_file(path: &Path) -> ForgeResult<String> {
    let source = read_script(path)?;
    let (_, report) = parse_and_analyze_source(&source)?;
    Ok(format_analysis_report(&report))
}

fn usage(program: &str) -> String {
    format!(
        "{DESCRIPTION}\n\nUsage:\n  {program} <command> [args]\n\nCommands:\n  run <file>       Validate and execute a Forge script\n  check <file>     Validate a script without executing it\n  explain <file>   Show inferred dimensions and output conversions\n  units            List supported built-in units\n  examples         List included example scripts and demo commands\n  version          Print the Forge version\n  help             Show this help text\n\nExamples:\n  {program} version\n  {program} units\n  {program} explain examples/axial_stress.forge\n  {program} run examples/beam_bending.forge\n  {program} check examples/dimension_error.forge"
    )
}

fn command_usage(program: &str, command: &str) -> String {
    format!("{}\n\n{}", command_error_hint(command), usage(program))
}

fn command_error_hint(command: &str) -> String {
    match command {
        "run" => "Usage for run:\n  forge run <file>",
        "check" => "Usage for check:\n  forge check <file>",
        "explain" => "Usage for explain:\n  forge explain <file>",
        "units" => "Usage for units:\n  forge units",
        "examples" => "Usage for examples:\n  forge examples",
        "version" => "Usage for version:\n  forge version",
        "help" => "Usage for help:\n  forge help",
        _ => "Run `forge help` for usage.",
    }
    .to_string()
}

fn format_units_listing() -> String {
    let mut rendered = String::from("Supported built-in units:\n");

    for category in UnitRegistry::categories() {
        rendered.push_str(&format!("\n{}:\n", category.title()));
        for unit in UnitRegistry::units_in_category(*category) {
            rendered.push_str(&format!("  {:<6} {}\n", unit.symbol, unit.dimension));
        }
    }

    rendered.trim_end().to_string()
}

fn format_analysis_report(report: &AnalysisReport) -> String {
    let mut rendered = String::from("Inferred dimensions:\n");

    if report.variables.is_empty() {
        rendered.push_str("  (none)\n");
    } else {
        let width = report
            .variables
            .iter()
            .map(|variable| variable.name.len())
            .max()
            .unwrap_or(0);
        for variable in &report.variables {
            rendered.push_str(&format!(
                "  {:width$}  {}\n",
                variable.name,
                variable.dimension,
                width = width
            ));
        }
    }

    rendered.push_str("\nOutputs:\n");
    if report.outputs.is_empty() {
        rendered.push_str("  (none)");
    } else {
        for output in &report.outputs {
            let status = if output.compatible {
                "compatible"
            } else {
                "incompatible"
            };
            match &output.as_unit {
                Some(unit) => rendered.push_str(&format!(
                    "  print {} as {}  {} ({})\n",
                    output.expression, unit, status, output.dimension
                )),
                None => rendered.push_str(&format!(
                    "  print {}  {} ({})\n",
                    output.expression, status, output.dimension
                )),
            }
        }
        rendered = rendered.trim_end().to_string();
    }

    rendered
}

fn format_examples_listing(program: &str) -> String {
    format!(
        "Included examples:\n  axial_stress.forge       axial stress from force and area\n  beam_bending.forge       bending stress from moment and section inertia\n  pressure_vessel.forge    thin-wall hoop stress estimate\n  power_torque.forge       torque and rotational rate converted to power\n  reynolds_number.forge    dimensionless Reynolds number\n  dimension_error.forge    intentional unit mistake for diagnostics\n\nSuggested commands:\n  {program} explain examples/axial_stress.forge\n  {program} run examples/pressure_vessel.forge\n  {program} run examples/power_torque.forge\n  {program} check examples/dimension_error.forge"
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
    let _invoked = args.next();
    let program = "forge".to_string();

    let Some(command) = args.next() else {
        eprintln!("{}", usage(&program));
        std::process::exit(2);
    };

    match command.as_str() {
        "help" | "-h" | "--help" => {
            if args.next().is_some() {
                eprintln!(
                    "The 'help' command does not take arguments.\n\n{}",
                    command_usage(&program, "help")
                );
                std::process::exit(2);
            }
            println!("{}", usage(&program));
        }
        "run" => {
            let Some(path) = args.next() else {
                eprintln!(
                    "Missing file path for 'run' command.\n\n{}",
                    command_usage(&program, "run")
                );
                std::process::exit(2);
            };

            if args.next().is_some() {
                eprintln!(
                    "Too many arguments for 'run' command.\n\n{}",
                    command_usage(&program, "run")
                );
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
                    command_usage(&program, "check")
                );
                std::process::exit(2);
            };

            if args.next().is_some() {
                eprintln!(
                    "Too many arguments for 'check' command.\n\n{}",
                    command_usage(&program, "check")
                );
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
        "explain" => {
            let Some(path) = args.next() else {
                eprintln!(
                    "Missing file path for 'explain' command.\n\n{}",
                    command_usage(&program, "explain")
                );
                std::process::exit(2);
            };

            if args.next().is_some() {
                eprintln!(
                    "Too many arguments for 'explain' command.\n\n{}",
                    command_usage(&program, "explain")
                );
                std::process::exit(2);
            }

            match explain_file(Path::new(&path)) {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("{}", format_cli_error(Path::new(&path), &error));
                    std::process::exit(1);
                }
            }
        }
        "units" => {
            if args.next().is_some() {
                eprintln!(
                    "The 'units' command does not take arguments.\n\n{}",
                    command_usage(&program, "units")
                );
                std::process::exit(2);
            }
            println!("{}", format_units_listing());
        }
        "examples" => {
            if args.next().is_some() {
                eprintln!(
                    "The 'examples' command does not take arguments.\n\n{}",
                    command_usage(&program, "examples")
                );
                std::process::exit(2);
            }
            println!("{}", format_examples_listing(&program));
        }
        "version" => {
            if args.next().is_some() {
                eprintln!(
                    "The 'version' command does not take arguments.\n\n{}",
                    command_usage(&program, "version")
                );
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
