use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_forge")
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(binary_path())
        .args(args)
        .output()
        .expect("failed to run forge binary")
}

fn run_cli_in_dir(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    Command::new(binary_path())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run forge binary")
}

fn run_script(name: &str) -> std::process::Output {
    let path = example_path(name);
    run_cli(&["run", path.to_string_lossy().as_ref()])
}

fn unique_temp_script_path(stem: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("forge_cli_{stem}_{timestamp}.forge"))
}

fn write_temp_script(stem: &str, source: &str) -> PathBuf {
    let path = unique_temp_script_path(stem);
    fs::write(&path, source).expect("failed to write temporary script");
    path
}

fn unique_temp_dir_path(stem: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("forge_cli_{stem}_{timestamp}"))
}

fn create_temp_dir(stem: &str) -> PathBuf {
    let path = unique_temp_dir_path(stem);
    fs::create_dir(&path).expect("failed to create temporary directory");
    path
}

#[test]
fn runs_axial_stress_example() {
    let output = run_script("axial_stress.forge");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "40 MPa");
}

#[test]
fn runs_hydrostatic_pressure_example() {
    let output = run_script("hydrostatic_pressure.forge");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "24.525 kPa");
}

#[test]
fn runs_kinetic_energy_example() {
    let output = run_script("kinetic_energy.forge");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "240 kJ");
}

#[test]
fn runs_unit_conversion_example() {
    let output = run_script("unit_conversion.forge");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "2.5 m");
}

#[test]
fn runs_stress_print_variants_example() {
    let output = run_script("stress_print_variants.forge");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "40000000 [L^-1 M T^-2]\n40 MPa"
    );
}

#[test]
fn runs_new_engineering_examples() {
    let cases = [
        ("beam_bending.forge", "32.1429 MPa"),
        ("pressure_vessel.forge", "45 MPa"),
        ("power_torque.forge", "2400 N*m\n72 kW"),
        ("shaft_power_rpm.forge", "47.1239 kW"),
        ("heat_energy.forge", "292.6 kJ"),
        ("imperial_pressure.forge", "861.845 kPa\n8.61845 bar"),
        ("fluid_volume_flow.forge", "0.4 L/s\n400 mL/s"),
        ("reynolds_number.forge", "59880"),
    ];

    for (script, expected) in cases {
        let output = run_script(script);
        assert!(
            output.status.success(),
            "{script} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
    }
}

#[test]
fn fails_for_invalid_dimensions_example() {
    let output = run_script("invalid_dimensions.forge");
    assert!(!output.status.success(), "script should fail");
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    let script = example_path("invalid_dimensions.forge");
    assert!(stderr.contains("error: Cannot add incompatible quantities."));
    assert!(stderr.contains(&format!("{}:4:1", script.display())));
    assert!(stderr.contains("4 | bad = length + time"));
    assert!(stderr.contains("^"));
    assert!(stderr.contains("= Left operand dimension: [L]"));
    assert!(stderr.contains("= Right operand dimension: [T]"));
    assert!(stderr.contains("= help: addition and subtraction require matching dimensions"));
}

#[test]
fn check_command_succeeds_for_valid_script() {
    let script = example_path("axial_stress.forge");
    let output = run_cli(&["check", script.to_string_lossy().as_ref()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Check passed:"));
}

#[test]
fn run_and_check_share_same_dimensional_error_diagnostic() {
    let script = example_path("invalid_dimensions.forge");
    let script_arg = script.to_string_lossy().into_owned();
    let run_output = run_cli(&["run", &script_arg]);
    let check_output = run_cli(&["check", &script_arg]);

    assert_eq!(run_output.status.code(), Some(1));
    assert_eq!(check_output.status.code(), Some(1));

    let run_stderr = String::from_utf8_lossy(&run_output.stderr);
    let check_stderr = String::from_utf8_lossy(&check_output.stderr);
    assert_eq!(run_stderr, check_stderr);
}

#[test]
fn check_reports_syntax_error_with_file_line_and_column() {
    let script = write_temp_script("invalid_syntax", "force 12 kN\n");
    let script_arg = script.to_string_lossy().into_owned();
    let output = run_cli(&["check", &script_arg]);
    let _ = fs::remove_file(&script);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: Invalid syntax: expected '=' after variable name in assignment.")
    );
    assert!(stderr.contains("Found number '12'."));
    assert!(stderr.contains(&format!("{}:1:7", script.display())));
    assert!(stderr.contains("1 | force 12 kN"));
    assert!(stderr.contains("^"));
    assert!(stderr.contains("= help: check the statement structure and unit expression syntax"));
}

#[test]
fn check_reports_unknown_unit_with_file_line_and_column() {
    let script = write_temp_script("unknown_unit", "mass = 10 slug\n");
    let script_arg = script.to_string_lossy().into_owned();
    let output = run_cli(&["check", &script_arg]);
    let _ = fs::remove_file(&script);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: Unknown unit 'slug'."));
    assert!(stderr.contains("Supported units are:"));
    assert!(stderr.contains("cm"));
    assert!(stderr.contains("kW"));
    assert!(stderr.contains(&format!("{}:1:1", script.display())));
    assert!(stderr.contains("1 | mass = 10 slug"));
    assert!(stderr.contains("^"));
    assert!(stderr.contains("= help: run `forge units` to see supported built-in units"));
}

#[test]
fn units_command_lists_grouped_registry() {
    let output = run_cli(&["units"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Supported built-in units:"));
    assert!(stdout.contains("Length:"));
    assert!(stdout.contains("  cm"));
    assert!(stdout.contains("Pressure / Stress:"));
    assert!(stdout.contains("  GPa"));
    assert!(stdout.contains("  psi"));
    assert!(stdout.contains("Volume:"));
    assert!(stdout.contains("  L"));
    assert!(stdout.contains("Power:"));
    assert!(stdout.contains("  kW"));
    assert!(stdout.contains("Temperature:"));
    assert!(stdout.contains("  K"));
    assert!(stdout.contains("Angle / Rotation:"));
    assert!(stdout.contains("  rpm"));
}

#[test]
fn examples_command_lists_demo_scripts() {
    let output = run_cli(&["examples"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Included examples:"));
    assert!(stdout.contains("axial_stress.forge"));
    assert!(stdout.contains("heat_energy.forge"));
    assert!(stdout.contains("shaft_power_rpm.forge"));
    assert!(stdout.contains("pressure_vessel.forge"));
    assert!(stdout.contains("Suggested commands:"));
    assert!(stdout.contains("forge new stress-check"));
    assert!(stdout.contains("forge run examples/shaft_power_rpm.forge"));
}

#[test]
fn explain_command_reports_inferred_dimensions() {
    let script = example_path("axial_stress.forge");
    let output = run_cli(&["explain", script.to_string_lossy().as_ref()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Inferred dimensions:"));
    assert!(stdout.contains("force   [L M T^-2]"));
    assert!(stdout.contains("area    [L^2]"));
    assert!(stdout.contains("stress  [L^-1 M T^-2]"));
    assert!(stdout.contains("Outputs:"));
    assert!(stdout.contains("print stress as MPa  compatible"));
}

#[test]
fn explain_command_shares_semantic_errors() {
    let script = example_path("dimension_error.forge");
    let output = run_cli(&["explain", script.to_string_lossy().as_ref()]);
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: Cannot add incompatible quantities."));
    assert!(stderr.contains("4 | badtotal = pressure + length"));
    assert!(stderr.contains("^"));
    assert!(stderr.contains("= Left operand dimension: [L^-1 M T^-2]"));
    assert!(stderr.contains("= Right operand dimension: [L]"));
}

#[test]
fn run_check_and_explain_share_dimension_error_diagnostic() {
    let script = example_path("dimension_error.forge");
    let script_arg = script.to_string_lossy().into_owned();
    let run_output = run_cli(&["run", &script_arg]);
    let check_output = run_cli(&["check", &script_arg]);
    let explain_output = run_cli(&["explain", &script_arg]);

    assert_eq!(run_output.status.code(), Some(1));
    assert_eq!(check_output.status.code(), Some(1));
    assert_eq!(explain_output.status.code(), Some(1));

    let run_stderr = String::from_utf8_lossy(&run_output.stderr);
    let check_stderr = String::from_utf8_lossy(&check_output.stderr);
    let explain_stderr = String::from_utf8_lossy(&explain_output.stderr);
    assert_eq!(run_stderr, check_stderr);
    assert_eq!(check_stderr, explain_stderr);
}

#[test]
fn invalid_command_shows_usage() {
    let output = run_cli(&["launch", "examples/axial_stress.forge"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command 'launch'"));
    assert!(stderr.contains("Forge is a unit-safe engineering worksheet language"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn help_command_shows_usage() {
    let output = run_cli(&["help"]);
    assert!(output.status.success(), "help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Forge is a unit-safe engineering worksheet language"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("run <file>"));
    assert!(stdout.contains("check <file>"));
    assert!(stdout.contains("explain <file>"));
    assert!(stdout.contains("new <project-name>"));
    assert!(stdout.contains("units"));
    assert!(stdout.contains("examples"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("help"));
    assert!(stdout.contains("Examples:"));
    assert!(stdout.contains("forge new stress-check"));
    assert!(stdout.contains("forge run examples/beam_bending.forge"));
}

#[test]
fn long_help_flag_shows_usage() {
    let output = run_cli(&["--help"]);
    assert!(output.status.success(), "--help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Forge is a unit-safe engineering worksheet language"));
    assert!(stdout.contains("forge explain examples/axial_stress.forge"));
}

#[test]
fn run_command_requires_file_path() {
    let output = run_cli(&["run"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing file path for 'run' command"));
    assert!(stderr.contains("Usage for run:"));
    assert!(stderr.contains("forge run <file>"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn run_command_rejects_too_many_arguments() {
    let output = run_cli(&["run", "examples/axial_stress.forge", "extra"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Too many arguments for 'run' command"));
    assert!(stderr.contains("Usage for run:"));
}

#[test]
fn new_command_creates_starter_project() {
    let root = create_temp_dir("new_project");
    let output = run_cli_in_dir(&["new", "stress-check"], &root);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let project_dir = root.join("stress-check");
    let worksheet =
        fs::read_to_string(project_dir.join("main.forge")).expect("starter worksheet should exist");
    let readme =
        fs::read_to_string(project_dir.join("README.md")).expect("starter README should exist");

    assert!(worksheet.contains("# stress-check"));
    assert!(worksheet.contains("force = 12 kN"));
    assert!(worksheet.contains("print stress as MPa"));
    assert!(readme.contains("forge run main.forge"));
    assert!(readme.contains("forge explain main.forge"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created Forge worksheet project: stress-check"));
    assert!(stdout.contains("cd stress-check"));
    assert!(stdout.contains("forge run main.forge"));

    let run_output = run_cli_in_dir(&["run", "main.forge"], &project_dir);
    assert!(
        run_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run_output.stdout).trim(), "40 MPa");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn new_command_requires_project_name() {
    let output = run_cli(&["new"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing project name for 'new' command"));
    assert!(stderr.contains("forge new <project-name>"));
}

#[test]
fn new_command_rejects_too_many_arguments() {
    let output = run_cli(&["new", "stress-check", "extra"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Too many arguments for 'new' command"));
    assert!(stderr.contains("Usage for new:"));
}

#[test]
fn new_command_rejects_invalid_project_names() {
    let root = create_temp_dir("invalid_project_names");
    let invalid_names = [
        "",
        ".",
        "..",
        "bad/name",
        "bad\\name",
        "bad:name",
        "bad*name",
        "bad?name",
        "bad\"name",
        "bad<name",
        "bad>name",
        "bad|name",
    ];

    for name in invalid_names {
        let output = run_cli_in_dir(&["new", name], &root);
        assert!(!output.status.success(), "{name:?} should fail");
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("error: Invalid project name"),
            "stderr for {name:?}: {stderr}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn new_command_refuses_to_overwrite_existing_directory() {
    let root = create_temp_dir("existing_project");
    let project_dir = root.join("stress-check");
    fs::create_dir(&project_dir).expect("existing project dir should be created");
    fs::write(project_dir.join("main.forge"), "print 1\n").expect("sentinel should be written");

    let output = run_cli_in_dir(&["new", "stress-check"], &root);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("target directory already exists"));

    let sentinel =
        fs::read_to_string(project_dir.join("main.forge")).expect("sentinel should still exist");
    assert_eq!(sentinel, "print 1\n");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn argumentless_commands_reject_arguments() {
    for command in ["units", "examples", "version", "help"] {
        let output = run_cli(&[command, "extra"]);
        assert!(!output.status.success(), "{command} should fail");
        assert_eq!(output.status.code(), Some(2));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(&format!("The '{command}' command does not take arguments.")),
            "stderr for {command}: {stderr}"
        );
    }
}
