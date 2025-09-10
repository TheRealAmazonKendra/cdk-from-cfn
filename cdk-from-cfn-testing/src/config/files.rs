// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::env::var;
use std::fs::{create_dir_all, read_to_string, remove_dir, remove_dir_all, write, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{Paths, Scope};

pub struct Files;

impl Files {
    fn read(path: &Path) -> Result<String, String> {
        read_to_string(path).map_err(|e| format!("Failed to read file {}: {}", path.display(), e))
    }

    pub fn write(path: &Path, content: &str) -> Result<(), String> {
        Self::create_parent_dirs(path)?;
        write(path, content).map_err(|e| format!("Failed to write file {}: {}", path.display(), e))
    }

    pub fn create_parent_dirs(path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }
        Ok(())
    }

    pub fn cleanup_directory(test_working_dir: &Path) -> Result<(), String> {
        remove_dir_all(test_working_dir).map_err(|e| {
            format!(
                "Failed to remove directory {}: {}",
                test_working_dir.display(),
                e
            )
        })?;

        let mut parent = test_working_dir.parent();
        while let Some(dir) = parent {
            if dir
                .read_dir()
                .map_or(true, |mut entries| entries.next().is_none())
            {
                remove_dir(dir).ok();
                parent = dir.parent();
            } else {
                break;
            }
            if let Some(name) = dir.file_name() {
                if name == Paths::ACTUAL_DIR || name == Paths::TESTING_DIR {
                    break;
                }
            }
        }
        Ok(())
    }

    fn open_zip_archive() -> Result<zip::ZipArchive<File>, String> {
        let zip_path = var("END_TO_END_SNAPSHOTS")
            .map_err(|_| "END_TO_END_SNAPSHOTS environment variable not set".to_string())?;
        let file = File::open(&zip_path)
            .map_err(|_| format!("Expected zip file not found: {}", zip_path))?;
        zip::ZipArchive::new(file).map_err(|_| format!("Failed to read zip file: {}", zip_path))
    }

    pub fn load_from_zip(zip_path: &str) -> Result<String, String> {
        let mut archive = Self::open_zip_archive()?;
        let mut file = archive
            .by_name(zip_path)
            .map_err(|_| format!("File not found in zip: {}", zip_path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file from zip: {}", e))?;
        Ok(contents)
    }

    pub fn load_case_template_from_zip(test_name: &str) -> Result<String, String> {
        Self::load_from_zip(&Paths::zip_case_path(test_name, Paths::TEMPLATE).to_string_lossy())
    }

    pub fn load_dependency_template_from_zip(test_name: &str) -> Result<String, String> {
        Self::load_from_zip(
            &Paths::zip_case_path(test_name, Paths::DEPENDENCY_TEMPLATE).to_string_lossy(),
        )
    }

    pub fn load_expected_stack_from_zip(test_name: &str, lang: &str) -> Result<String, String> {
        let mut archive = Self::open_zip_archive()?;
        let dir_prefix = format!(
            "{}/",
            Paths::zip_expected_dir(test_name, lang).to_string_lossy()
        );

        let stack_file = archive
            .file_names()
            .find(|name| name.starts_with(&dir_prefix) && !name.ends_with('/'))
            .ok_or_else(|| format!("No stack file found in directory {} in zip", dir_prefix))?
            .to_string();

        let mut file = archive
            .by_name(&stack_file)
            .map_err(|_| format!("File not found in zip: {}", stack_file))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file from zip: {}", e))?;
        Ok(contents)
    }

    pub fn load_actual_stack(scope: &Scope) -> Result<String, String> {
        let path = Paths::actual_stack_path(scope)?;
        Self::read(&path)
            .map_err(|e| format!("Failed to read actual output {}: {e}", path.display()))
    }

    #[cfg_attr(not(feature = "update-snapshots"), allow(dead_code))]
    pub fn load_expected_stack(test_name: &str, lang: &str) -> Result<String, String> {
        let expected_dir = Paths::expected_dir().join(test_name).join(lang);
        let file_path = Self::find_single_file_recursive(&expected_dir)?;
        Self::read(&file_path)
    }

    #[cfg_attr(not(feature = "update-snapshots"), allow(dead_code))]
    fn find_single_file_recursive(dir: &Path) -> Result<PathBuf, String> {
        fn find_file_recursive(path: &Path) -> Option<PathBuf> {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        return Some(entry_path);
                    } else if entry_path.is_dir() {
                        if let Some(file) = find_file_recursive(&entry_path) {
                            return Some(file);
                        }
                    }
                }
            }
            None
        }

        find_file_recursive(dir)
            .ok_or_else(|| format!("No file found in directory: {}", dir.display()))
    }

    pub fn write_expected_stack(
        scope: &Scope,
        stack_name: &str,
        content: &str,
    ) -> Result<(), String> {
        let path = Paths::expected_stack_path(scope, stack_name);
        Self::write(&path, content)
    }

    pub fn write_actual_stack(scope: &Scope, content: &str) -> Result<(), String> {
        let path = Paths::actual_stack_path(scope)?;
        Self::write(&path, content)
    }

    pub fn load_actual_synthesized_template(
        scope: &Scope,
        stack_name: &str,
    ) -> Result<String, String> {
        let path = Paths::synthesized_template_path(scope, stack_name);
        Self::read(&path).map_err(|e| {
            format!(
                "Failed to read synthesized template form {}: {e}",
                path.display()
            )
        })
    }

    pub fn cleanup_test(scope: &Scope) -> Result<(), String> {
        let test_dir = Paths::actual_dir_path(scope);
        Self::cleanup_directory(&test_dir)
    }



    pub fn copy_file(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
        std::fs::copy(from, to).map(|_| ()).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {}",
                from.display(),
                to.display(),
                e
            )
        })
    }

    pub fn copy(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
        Self::copy_file(from, to)
    }

    pub fn create_dir_all(path: &std::path::Path) -> Result<(), String> {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))
    }

    pub fn create_file(path: &std::path::Path) -> Result<std::fs::File, String> {
        Self::create_parent_dirs(path)?;
        std::fs::File::create(path).map_err(|e| format!("Failed to create file {}: {}", path.display(), e))
    }

    pub fn remove_file(path: &std::path::Path) -> Result<(), String> {
        std::fs::remove_file(path).map_err(|e| format!("Failed to remove file {}: {}", path.display(), e))
    }

    pub fn write_diff(path: &std::path::Path, content: &str) -> Result<(), String> {
        Self::write(path, content)?;
        println!("  🪄  Updated Stack.diff: {}", path.display());
        Ok(())
    }

    pub fn load_acceptable_diff(test_name: &str) -> Result<String, String> {
        Self::load_from_zip(
            &Paths::zip_case_path(test_name, "Stack.diff").to_string_lossy(),
        )
    }

    pub fn setup_temp_directory(source_dir: &std::path::Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::process::{id, Command};
        use std::env::temp_dir;
        
        static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
        
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = temp_dir().join(format!("cdk_test_{}_{}", id(), counter));
        Self::create_dir_all(&temp_dir)?;

        // Clean build artifacts from source before copying
        Command::new("rm")
            .args(["-rf", &format!("{}/bin", source_dir.to_string_lossy())])
            .output()?;
        Command::new("rm")
            .args(["-rf", &format!("{}/obj", source_dir.to_string_lossy())])
            .output()?;

        Command::new("cp")
            .args([
                "-a",
                &format!("{}/.", source_dir.to_string_lossy()),
                &temp_dir.to_string_lossy(),
            ])
            .output()?;

        Ok(temp_dir)
    }

    pub fn cleanup_temp_directory(
        temp_dir: &std::path::Path,
        original_dir: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let cdk_out_src = temp_dir.join(Paths::CDK_OUT_DIR);
        let cdk_out_dst = original_dir.join(Paths::CDK_OUT_DIR);

        if cdk_out_src.exists() {
            if cdk_out_dst.exists() {
                Self::cleanup_directory(&cdk_out_dst).ok();
            }
            Command::new("cp")
                .args([
                    "-r",
                    &cdk_out_src.to_string_lossy(),
                    &cdk_out_dst.to_string_lossy(),
                ])
                .output()?;
        }

        Self::cleanup_directory(temp_dir).ok();
        Ok(())
    }
}