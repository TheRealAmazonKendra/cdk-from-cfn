// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};

use super::{Language, Scope};

pub struct Paths;

impl Paths {
    pub const TESTING_DIR: &'static str = "tests";
    pub const ACTUAL_DIR: &'static str = "actual";
    pub const CASES_DIR: &'static str = "cases";
    pub const EXPECTED_DIR: &'static str = "expected";
    pub const CDK_STACK_SYNTH_TEST_DIR: &'static str = "cdk_stack_synth";

    pub const TEMPLATE: &'static str = "template.json";
    pub const CDK_OUT_DIR: &'static str = "cdk.out";

    pub const DEPENDENCY_TEMPLATE: &'static str = "dependency_stack_template.json";
    pub const E2E_TAG: &'static str = "cdk-from-cfn-e2e-test";
    pub const E2E_DEPENDENCY_TAG: &'static str = "cdk-from-cfn-e2e-test-dependency-stack";

    pub fn testing_dir() -> PathBuf {
        PathBuf::from(Self::TESTING_DIR)
    }

    pub fn expected_dir() -> PathBuf {
        Self::testing_dir().join(Self::EXPECTED_DIR)
    }

    pub fn actual_dir() -> PathBuf {
        Self::testing_dir().join(Self::ACTUAL_DIR)
    }

    fn test_case_actual(normalized_test_name: &str) -> PathBuf {
        Self::actual_dir().join(normalized_test_name)
    }

    pub fn synthesized_template_path(scope: &Scope, stack_name: &str) -> PathBuf {
        Self::actual_dir_path(scope)
            .join(Self::CDK_OUT_DIR)
            .join(format!("{}.{}", Self::e2e_name(stack_name), Self::TEMPLATE))
    }

    pub fn e2e_name(name: &str) -> String {
        format!("{}-{}", Self::E2E_TAG, name)
    }

    pub fn actual_dir_path(scope: &Scope) -> PathBuf {
        Self::test_case_actual(&scope.normalized)
    }

    pub fn actual_stack_path(scope: &Scope) -> Result<PathBuf, String> {
        Self::stack_path(&Self::actual_dir_path(scope), scope)
    }

    pub fn expected_stack_path(scope: &Scope, stack_name: &str) -> PathBuf {
        let filename = Language::stack_filename(&scope.lang, stack_name);
        Self::expected_dir()
            .join(&scope.test)
            .join(&scope.lang)
            .join(filename)
    }

    fn stack_path(base_dir: &Path, scope: &Scope) -> Result<PathBuf, String> {
        let expected_dir = Self::expected_dir().join(&scope.test).join(&scope.lang);
        let file = Self::resolve_stack_file(&expected_dir)?;
        Ok(base_dir.join(file))
    }

    fn resolve_stack_file(dir_path: &Path) -> Result<PathBuf, String> {
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

        if let Some(file_path) = find_file_recursive(dir_path) {
            Ok(file_path
                .strip_prefix(dir_path)
                .unwrap_or(&file_path)
                .to_path_buf())
        } else {
            Err(format!(
                "No stack file found in directory: {}",
                dir_path.display()
            ))
        }
    }

    pub fn zip_expected_dir(test: &str, lang: &str) -> PathBuf {
        PathBuf::from(Self::EXPECTED_DIR).join(test).join(lang)
    }

    pub fn zip_case_path(test: &str, file: &str) -> PathBuf {
        PathBuf::from(Self::CASES_DIR).join(test).join(file)
    }

    pub fn cdk_path() -> Result<PathBuf, String> {
        Ok(Self::project_root()?
            .join("target/tmp")
            .join("node_modules")
            .join(".bin")
            .join("cdk"))
    }

    pub fn project_root() -> Result<PathBuf, String> {
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
    }

    pub fn app(scope: &Scope) -> PathBuf {
        Self::actual_dir_path(scope).join(Language::app_name(&scope.lang))
    }

    pub fn dependency_name(name: &str) -> String {
        format!("{}-{}", Self::E2E_DEPENDENCY_TAG, name)
    }

    pub fn boilerplate_dir(lang: &str) -> PathBuf {
        Self::fixtures_dir(lang).join("boilerplate")
    }

    pub fn setup_script(lang: &str) -> Result<PathBuf, String> {
        Ok(Self::project_root()?
            .join(Self::fixtures_dir(lang))
            .join("setup-and-synth.sh"))
    }

    pub fn case_path(test: &str, file: &str) -> PathBuf {
        Self::testing_dir()
            .join(Self::CASES_DIR)
            .join(test)
            .join(file)
    }

    pub fn fixtures_dir(lang: &str) -> PathBuf {
        Self::testing_dir()
            .join(Self::CDK_STACK_SYNTH_TEST_DIR)
            .join("install")
            .join("fixtures")
            .join(lang)
    }

    pub fn package_json_src() -> PathBuf {
        Self::fixtures_dir("typescript")
            .join("boilerplate")
            .join("package.json")
    }

    pub fn package_json_target() -> PathBuf {
        Path::new("target/tmp").join("package.json")
    }
}
