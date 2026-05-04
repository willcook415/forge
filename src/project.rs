//! Starter project scaffolding for `forge new`.

use std::fs;
use std::path::Path;

use crate::error::{ForgeError, ForgeResult};

const INVALID_PROJECT_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// Creates a starter Forge worksheet project in the current working directory.
pub fn create_project(name: &str, cwd: &Path) -> ForgeResult<()> {
    validate_project_name(name)?;

    let project_dir = cwd.join(name);
    if project_dir.exists() {
        return Err(ForgeError::new(format!(
            "Cannot create project '{}': target directory already exists.",
            name
        )));
    }

    fs::create_dir(&project_dir).map_err(|error| {
        ForgeError::new(format!(
            "Failed to create project directory '{}': {error}",
            project_dir.display()
        ))
    })?;

    write_project_file(&project_dir.join("main.forge"), &starter_worksheet(name))?;
    write_project_file(&project_dir.join("README.md"), &starter_readme(name))?;

    Ok(())
}

fn validate_project_name(name: &str) -> ForgeResult<()> {
    if name.trim().is_empty() {
        return Err(invalid_project_name(name, "project name cannot be empty"));
    }

    if name == "." || name == ".." {
        return Err(invalid_project_name(
            name,
            "project name must be a directory name, not '.' or '..'",
        ));
    }

    if let Some(ch) = name.chars().find(|ch| INVALID_PROJECT_CHARS.contains(ch)) {
        return Err(invalid_project_name(
            name,
            &format!("project name cannot contain '{ch}'"),
        ));
    }

    Ok(())
}

fn invalid_project_name(name: &str, reason: &str) -> ForgeError {
    ForgeError::new(format!("Invalid project name '{}': {reason}.", name))
}

fn write_project_file(path: &Path, contents: &str) -> ForgeResult<()> {
    fs::write(path, contents).map_err(|error| {
        ForgeError::new(format!(
            "Failed to write starter file '{}': {error}",
            path.display()
        ))
    })
}

fn starter_worksheet(name: &str) -> String {
    format!(
        "# {name}\nforce = 12 kN\narea = 300 mm^2\nstress = force / area\nprint stress as MPa\n"
    )
}

fn starter_readme(name: &str) -> String {
    format!(
        "# {name}\n\nA starter Forge worksheet project.\n\n```bash\nforge run main.forge\nforge check main.forge\nforge explain main.forge\n```\n"
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::project::create_project;

    fn temp_dir() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("forge_project_unit_{timestamp}"))
    }

    #[test]
    fn creates_starter_project() {
        let root = temp_dir();
        fs::create_dir(&root).expect("temp root should be created");

        create_project("stress-check", &root).expect("project should be created");
        let worksheet = fs::read_to_string(root.join("stress-check").join("main.forge"))
            .expect("worksheet should exist");
        let readme = fs::read_to_string(root.join("stress-check").join("README.md"))
            .expect("README should exist");

        assert!(worksheet.contains("# stress-check"));
        assert!(worksheet.contains("print stress as MPa"));
        assert!(readme.contains("forge run main.forge"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_project_name() {
        let root = temp_dir();
        fs::create_dir(&root).expect("temp root should be created");

        let error = create_project("bad/name", &root).expect_err("name should fail");
        assert!(error.message.contains("Invalid project name"));

        let _ = fs::remove_dir_all(root);
    }
}
