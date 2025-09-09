// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::string::{String, ToString};

use crate::cdk_stack_synth::{Files, Paths};
use cdk_from_cfn::testing::{Language, Scope, Stack, StackValidator, Stacks, Templates};

use crate::cdk_stack_synth::synth::{SkipList, Template};

#[allow(dead_code)]
pub trait CdkFromCfnStack {
    fn generate_stack(template: &str, lang: &str, stack_name: &str) -> Vec<u8>;
    fn synth(&self, skip_list: &SkipList, region: &str) -> Result<(), Box<dyn Error>>;
    fn setup_working_directory(&self) -> Result<(PathBuf, bool), Box<dyn Error>>;
    fn run_cdk_synth(&self, working_dir: &Path, region: &str) -> Result<Output, Box<dyn Error>>;
    fn handle_synth_result(&self, output: Output) -> Result<(), Box<dyn Error>>;
}

impl CdkFromCfnStack for Stack {
    fn generate_stack(template: &str, lang: &str, stack_name: &str) -> Vec<u8> {
        let cdk_from_cfn_path = std::env::var("CDK_FROM_CFN_PATH")
            .unwrap_or_else(|_| "./target/debug/cdk-from-cfn".to_string());
        let mut child = match Command::new(&cdk_from_cfn_path)
            .args([
                "-",
                "--language",
                Language::lang_arg(lang),
                "--stack-name",
                stack_name,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => panic!("Failed to execute cdk-from-cfn: {}", e),
        };

        if let Some(stdin) = child.stdin.as_mut() {
            if let Err(e) = stdin.write_all(template.as_bytes()) {
                panic!("Failed to write to stdin: {}", e);
            }
        }

        let output = child.wait_with_output().expect("Failed to read output");

        if !output.status.success() {
            panic!(
                "  ❌ cdk-from-cfn failed: {}: {:?}",
                output.status.code().expect("Unknown Error"),
                output.stderr
            );
        }

        output.stdout
    }

    fn synth(&self, skip_list: &SkipList, region: &str) -> Result<(), Box<dyn Error>> {
        if skip_list.should_skip(&self.scope, "CDK synth") {
            return Ok(());
        }

        let (working_dir, cleanup_temp) = self.setup_working_directory()?;
        let output = self.run_cdk_synth(&working_dir, region)?;

        if cleanup_temp {
            Files::cleanup_temp_directory(
                &working_dir,
                &cdk_from_cfn::testing::Paths::actual_dir_path(&self.scope),
            )?;
        }

        self.handle_synth_result(output)
    }

    fn setup_working_directory(&self) -> Result<(PathBuf, bool), Box<dyn Error>> {
        let working_dir = cdk_from_cfn::testing::Paths::actual_dir_path(&self.scope);

        if self.scope.lang == "csharp" {
            Ok((Files::setup_temp_directory(&working_dir)?, true))
        } else {
            Ok((working_dir, false))
        }
    }

    fn run_cdk_synth(&self, working_dir: &Path, region: &str) -> Result<Output, Box<dyn Error>> {
        let setup_script_path = Paths::setup_script(&self.scope.lang)?;

        Command::new("bash")
            .arg(&setup_script_path)
            .current_dir(working_dir)
            .env("CDK_DEFAULT_REGION", region)
            .env("AWS_DEFAULT_REGION", region)
            .env("PROJECT_ROOT", Paths::project_root()?)
            .env("CDK_PATH", Paths::cdk_path()?)
            .env("CDK_FLAGS", "--no-version-reporting --no-path-metadata")
            .output()
            .map_err(|e| -> Box<dyn Error> {
                format!("Failed to execute CDK command: {}", e).into()
            })
    }

    fn handle_synth_result(&self, output: Output) -> Result<(), Box<dyn Error>> {
        if output.status.success() {
            Ok(())
        } else {
            if !output.stdout.is_empty() {
                println!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
            }
            Err("  ❌ CDK synthesis failed".into())
        }
    }
}

#[derive(Clone)]
pub struct CdkStackValidator {
    pub validator: StackValidator,
    should_synth: bool,
}

impl CdkStackValidator {
    pub fn from_scope(scope: &Scope, stack_name: &str, skip_list: &SkipList) -> Self {
        let stacks = Stacks::from(scope);
        let validator = StackValidator {
            scope: scope.clone(),
            expected_stack: stacks.expected.clone(),
            actual_stack: stacks.actual.clone(),
            stack_name: stack_name.to_string(),
        };
        let should_synth = skip_list.should_synth(scope);
        Self {
            validator,
            should_synth,
        }
    }

    pub fn stack_files_match_expected(&self) -> Result<(), String> {
        self.validator.actual_stack_files_match_expected()
    }

    pub fn cdk_out_matches_cfn_stack_file(&self) -> Result<(), String> {
        if self.should_synth {
            Template::validate(&self.validator.scope, &self.validator.stack_name)
                .map_err(|e| format!("  ❌ Template validation failed: {}", e))?
        }
        Ok(())
    }

    pub fn synthesized_apps_match_each_other(&self, skip_list: &SkipList) -> Result<(), String> {
        if !self.should_synth {
            return Ok(());
        }
        let templates = self.get_synthesized_templates_for_test(skip_list);
        if templates.len() <= 1 {
            return Ok(());
        }
        Template::compare_multiple_templates(&templates)?;
        eprintln!(
            "  ✨ All synthesized CDK Apps match for {}::{{{}}}",
            self.validator.scope.test,
            skip_list.get_synth_languages().join(", ")
        );
        Ok(())
    }

    fn get_synthesized_templates_for_test(&self, skip_list: &SkipList) -> HashMap<String, String> {
        let mut result = HashMap::new();
        let languages = skip_list.get_synth_languages();

        for lang in languages {
            let lang_scope = Scope::new(&self.validator.scope.test, &lang);
            if let Ok(content) = Templates::cdk(&lang_scope, &self.validator.stack_name)
                .and_then(|t| t.synthesized_template().map(|s| s.to_string()))
            {
                if !content.is_empty() {
                    result.insert(lang.clone(), content);
                }
            }
        }
        result
    }
}
