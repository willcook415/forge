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
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "240000 [L^2 M T^-2]"
    );
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
fn fails_for_invalid_dimensions_example() {
    let output = run_script("invalid_dimensions.forge");
    assert!(!output.status.success(), "script should fail");
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    let script = example_path("invalid_dimensions.forge");
    assert!(stderr.contains("error: Cannot add incompatible quantities."));
    assert!(stderr.contains(&format!("{}:4:1", script.display())));
    assert!(stderr.contains("= Left operand dimension: [L]"));
    assert!(stderr.contains("= Right operand dimension: [T]"));
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
    assert!(stderr.contains("error: Invalid syntax: expected '=' after variable name in assignment."));
    assert!(stderr.contains("Found number '12'."));
    assert!(stderr.contains(&format!("{}:1:7", script.display())));
}

#[test]
fn check_reports_unknown_unit_with_file_line_and_column() {
    let script = write_temp_script("unknown_unit", "length = 10 cm\n");
    let script_arg = script.to_string_lossy().into_owned();
    let output = run_cli(&["check", &script_arg]);
    let _ = fs::remove_file(&script);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: Unknown unit 'cm'."));
    assert!(stderr.contains("Supported units are: m, mm, s, kg, N, kN, Pa, kPa, MPa."));
    assert!(stderr.contains(&format!("{}:1:1", script.display())));
}

#[test]
fn invalid_command_shows_usage() {
    let output = run_cli(&["launch", "examples/axial_stress.forge"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command 'launch'"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn help_command_shows_usage() {
    let output = run_cli(&["help"]);
    assert!(output.status.success(), "help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("run <file>"));
    assert!(stdout.contains("check <file>"));
    assert!(stdout.contains("version"));
}

#[test]
fn run_command_requires_file_path() {
    let output = run_cli(&["run"]);
    assert!(!output.status.success(), "command should fail");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing file path for 'run' command"));
    assert!(stderr.contains("Usage:"));
}
