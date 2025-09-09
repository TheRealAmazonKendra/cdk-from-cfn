// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk_stack_synth::Paths;
use cdk_from_cfn::testing::{Files as BaseFiles, Paths as BasePaths};
use std::env::temp_dir;
use std::error::Error;
use std::fs::{copy, create_dir_all, remove_file, File};
use std::path::{Path, PathBuf};
use std::process::{id, Command};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Files;

impl Files {
    pub fn create_dir_all(path: &Path) -> Result<(), String> {
        create_dir_all(path)
            .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))
    }

    pub fn copy(from: &Path, to: &Path) -> Result<(), String> {
        copy(from, to).map(|_| ()).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {}",
                from.display(),
                to.display(),
                e
            )
        })
    }

    pub fn create_file(path: &Path) -> Result<File, String> {
        BaseFiles::create_parent_dirs(path)?;
        File::create(path).map_err(|e| format!("Failed to create file {}: {}", path.display(), e))
    }

    pub fn remove_file(path: &Path) -> Result<(), String> {
        remove_file(path).map_err(|e| format!("Failed to remove file {}: {}", path.display(), e))
    }

    pub fn write_diff(path: &Path, content: &str) -> Result<(), String> {
        BaseFiles::write(path, content)?;
        println!("  🪄  Updated Stack.diff: {}", path.display());
        Ok(())
    }

    pub fn load_acceptable_diff(test_name: &str) -> Result<String, String> {
        BaseFiles::load_from_zip(
            &BasePaths::zip_case_path(test_name, Paths::STACK_DIFF_FILE).to_string_lossy(),
        )
    }

    pub fn setup_temp_directory(source_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
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
        temp_dir: &Path,
        original_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let cdk_out_src = temp_dir.join(cdk_from_cfn::testing::Paths::CDK_OUT_DIR);
        let cdk_out_dst = original_dir.join(cdk_from_cfn::testing::Paths::CDK_OUT_DIR);

        if cdk_out_src.exists() {
            if cdk_out_dst.exists() {
                BaseFiles::cleanup_directory(&cdk_out_dst).ok();
            }
            Command::new("cp")
                .args([
                    "-r",
                    &cdk_out_src.to_string_lossy(),
                    &cdk_out_dst.to_string_lossy(),
                ])
                .output()?;
        }

        BaseFiles::cleanup_directory(temp_dir).ok();
        Ok(())
    }
}
