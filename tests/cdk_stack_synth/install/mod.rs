use cdk_from_cfn_testing::{Files, Paths as BasePaths, Scope};
use std::error::Error;
use std::process::Command;
use walkdir::WalkDir;

mod fixtures;

use fixtures::Writer;

pub struct Install {}

impl Install {
    pub fn shared() -> Result<(), Box<dyn Error>> {
        if BasePaths::cdk_path()?.exists() {
            return Ok(());
        }

        Files::create_dir_all(&std::path::Path::new("target/tmp"))?;
        Files::copy_file(&BasePaths::package_json_src(), &BasePaths::package_json_target())?;

        let output = Command::new("npm")
            .args(["install", "--silent"])
            .current_dir(std::path::Path::new("target/tmp"))
            .output()
            .map_err(|e| format!("Failed to run npm install: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "npm install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    pub fn app_file(scope: &Scope, stack_name: &str) -> Result<(), Box<dyn Error>> {
        Writer::write_app_file(scope, stack_name, scope.test == "ec2_encryption")
    }

    pub fn boilerplate_files(scope: &Scope) -> Result<(), Box<dyn Error>> {
        let boilerplate_dir = BasePaths::boilerplate_dir(&scope.lang);
        let dest_dir = BasePaths::actual_dir_path(scope);

        for entry in WalkDir::new(&boilerplate_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                let filename = entry
                    .file_name()
                    .to_str()
                    .ok_or_else(|| format!("Invalid filename: {:?}", entry.file_name()))?;

                Files::copy(entry.path(), &dest_dir.join(filename))?;
            }
        }
        Ok(())
    }
}
