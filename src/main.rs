//! Forge CLI entrypoint.

mod ast;
mod diagnostic;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod project;
mod semantic;
mod token;
mod units;

use ast::Program;
use std::path::Path;

use diagnostic::render_file_error;
use error::{ForgeError, ForgeResult};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use project::create_project;
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

fn usage(program: &str) -> String {
    format!(
        "{DESCRIPTION}\n\nUsage:\n  {program} <command> [args]\n\nCommands:\n  run <file>             Validate and execute a Forge script\n  check <file>           Validate a script without executing it\n  explain <file>         Show inferred dimensions and output conversions\n  new <project-name>     Create a starter Forge worksheet project\n  units                  List supported built-in units\n  examples               List included example scripts and demo commands\n  version                Print the Forge version\n  help                   Show this help text\n\nExamples:\n  {program} version\n  {program} units\n  {program} new stress-check\n  {program} explain examples/axial_stress.forge\n  {program} run examples/beam_bending.forge\n  {program} check examples/dimension_error.forge"
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
        "new" => "Usage for new:\n  forge new <project-name>",
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
        "Included examples:\n  axial_stress.forge        axial stress from force and area\n  beam_bending.forge        bending stress from moment and section inertia\n  pressure_vessel.forge     thin-wall hoop stress estimate\n  power_torque.forge        torque and rotational rate converted to power\n  shaft_power_rpm.forge     shaft power from torque and rpm\n  heat_energy.forge         heat energy from mass, heat capacity, and temperature rise\n  imperial_pressure.forge   pressure conversion from psi\n  fluid_volume_flow.forge   volumetric flow rate using litres\n  reynolds_number.forge     dimensionless Reynolds number\n  dimension_error.forge     intentional unit mistake for diagnostics\n\nSuggested commands:\n  {program} new stress-check\n  {program} explain examples/axial_stress.forge\n  {program} run examples/heat_energy.forge\n  {program} run examples/shaft_power_rpm.forge\n  {program} run examples/imperial_pressure.forge\n  {program} check examples/dimension_error.forge"
    )
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

            let path = Path::new(&path);
            let source = match read_script(path) {
                Ok(source) => source,
                Err(error) => {
                    eprintln!("{}", render_file_error(path, None, &error));
                    std::process::exit(1);
                }
            };

            match run_source(&source) {
                Ok(output) => {
                    for line in output {
                        println!("{line}");
                    }
                }
                Err(error) => {
                    eprintln!("{}", render_file_error(path, Some(&source), &error));
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

            let path = Path::new(&path);
            let source = match read_script(path) {
                Ok(source) => source,
                Err(error) => {
                    eprintln!("{}", render_file_error(path, None, &error));
                    std::process::exit(1);
                }
            };

            match check_source(&source) {
                Ok(()) => {
                    println!("Check passed: {}", path.display());
                }
                Err(error) => {
                    eprintln!("{}", render_file_error(path, Some(&source), &error));
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

            let path = Path::new(&path);
            let source = match read_script(path) {
                Ok(source) => source,
                Err(error) => {
                    eprintln!("{}", render_file_error(path, None, &error));
                    std::process::exit(1);
                }
            };

            match parse_and_analyze_source(&source)
                .map(|(_, report)| format_analysis_report(&report))
            {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("{}", render_file_error(path, Some(&source), &error));
                    std::process::exit(1);
                }
            }
        }
        "new" => {
            let Some(project_name) = args.next() else {
                eprintln!(
                    "Missing project name for 'new' command.\n\n{}",
                    command_usage(&program, "new")
                );
                std::process::exit(2);
            };

            if args.next().is_some() {
                eprintln!(
                    "Too many arguments for 'new' command.\n\n{}",
                    command_usage(&program, "new")
                );
                std::process::exit(2);
            }

            let cwd = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(error) => {
                    eprintln!("error: Failed to determine current directory: {error}");
                    std::process::exit(1);
                }
            };

            match create_project(&project_name, &cwd) {
                Ok(()) => {
                    println!(
                        "Created Forge worksheet project: {project_name}\n\nNext:\n  cd {project_name}\n  forge run main.forge\n  forge explain main.forge"
                    );
                }
                Err(error) => {
                    eprintln!("error: {}", error.message);
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
