// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use cdk_from_cfn::testing::{Language, Paths as BasePaths, Scope};
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

pub struct Paths;

impl Paths {
    pub const TARGET_TMP: &'static str = "target/tmp";
    pub const PACKAGE_JSON: &'static str = "package.json";
    pub const BOILERPLATE: &'static str = "boilerplate";
    pub const STACK_DIFF_FILE: &'static str = "Stack.diff";

    pub fn cdk_path() -> Result<PathBuf, String> {
        Ok(Self::project_root()?
            .join(Self::target_tmp_path())
            .join("node_modules")
            .join(".bin")
            .join("cdk"))
    }

    pub fn target_tmp_path() -> &'static Path {
        Path::new(Self::TARGET_TMP)
    }

    pub fn package_json_src() -> PathBuf {
        Self::fixtures_dir("typescript")
            .join(Self::BOILERPLATE)
            .join(Self::PACKAGE_JSON)
    }

    pub fn package_json_target() -> PathBuf {
        Self::target_tmp_path().join(Self::PACKAGE_JSON)
    }

    pub fn project_root() -> Result<PathBuf, String> {
        current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
    }

    pub fn app(scope: &Scope) -> PathBuf {
        BasePaths::actual_dir_path(scope).join(Language::app_name(&scope.lang))
    }

    pub fn dependency_name(name: &str) -> String {
        format!("{}-{}", BasePaths::E2E_DEPENDENCY_TAG, name)
    }

    pub fn boilerplate_dir(lang: &str) -> PathBuf {
        Self::fixtures_dir(lang).join(Self::BOILERPLATE)
    }

    pub fn setup_script(lang: &str) -> Result<PathBuf, String> {
        Ok(Self::project_root()?
            .join(Self::fixtures_dir(lang))
            .join("setup-and-synth.sh"))
    }

    pub fn case_path(test: &str, file: &str) -> PathBuf {
        BasePaths::testing_dir()
            .join(BasePaths::CASES_DIR)
            .join(test)
            .join(file)
    }

    pub fn fixtures_dir(lang: &str) -> PathBuf {
        BasePaths::testing_dir()
            .join(BasePaths::CDK_STACK_SYNTH_TEST_DIR)
            .join("install")
            .join("fixtures")
            .join(lang)
    }
}
