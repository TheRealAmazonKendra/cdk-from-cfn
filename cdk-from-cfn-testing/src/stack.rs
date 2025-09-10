// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::config::{Scope, Stacks, Templates};
use similar::{ChangeTag, TextDiff};

pub struct Stack {
    pub scope: Scope,
    pub expected_stack: String,
    pub actual_stack: String,
    pub stack_name: String,
}

impl Stack {
    pub fn new<F>(scope: Scope, stack_name: &str, generate_stack: F) -> Self
    where
        F: Fn(&str, &str, &str) -> Vec<u8>,
    {
        if let Templates::Cfn {
            original_template: template,
            ..
        } = Templates::cfn(&scope.test)
        {
            let output = generate_stack(&template, &scope.lang, stack_name);
            let content = String::from_utf8(output).unwrap();
            let Stacks { expected, actual } = Stacks::new(scope.clone(), &content, stack_name);

            Self {
                scope,
                expected_stack: expected.clone(),
                actual_stack: actual.clone(),
                stack_name: stack_name.to_string(),
            }
        } else {
            panic!(
                "Stack file for test could not be written to tests/actual: {}",
                scope.normalized
            )
        }
    }

    pub fn validator(&self) -> StackValidator {
        StackValidator::from_stack(self)
    }
}

/// Handles comparison of text-based outputs (stack files, code generation)
#[derive(Clone)]
pub struct StackValidator {
    pub scope: Scope,
    pub expected_stack: String,
    pub actual_stack: String,
    pub stack_name: String,
}

impl StackValidator {
    pub fn from_stack(stack: &Stack) -> Self {
        Self {
            scope: stack.scope.clone(),
            expected_stack: stack.expected_stack.clone(),
            actual_stack: stack.actual_stack.clone(),
            stack_name: stack.stack_name.clone(),
        }
    }

    /// Validate that the generated stack file matches expected output
    pub fn actual_stack_files_match_expected(&self) -> Result<(), String> {
        let result = self.compare_and_report();

        // Update snapshots if requested
        if cfg!(feature = "update-snapshots") {
            // Always return Ok when updating snapshots to continue with other validations
            Ok(())
        } else {
            result
        }
    }

    fn compare_and_report(&self) -> Result<(), String> {
        if self.expected_stack == self.actual_stack {
            eprintln!(
                "  ✨ Stack files match expected for {} ({})",
                self.scope.test, self.scope.lang
            );
            Ok(())
        } else {
            self.print_diff();
            Err(format!(
                "  ❌ Stack file mismatch for {} ({})",
                self.scope.test, self.scope.lang
            ))
        }
    }

    fn print_diff(&self) {
        let diff = TextDiff::from_lines(&self.expected_stack, &self.actual_stack);
        let differences: Vec<String> = diff
            .iter_all_changes()
            .filter_map(|change| match change.tag() {
                ChangeTag::Delete => Some(format!("- {}", change.value().trim_end())),
                ChangeTag::Insert => Some(format!("+ {}", change.value().trim_end())),
                ChangeTag::Equal => None,
            })
            .collect();

        let output = format!(
            "❌ Test {} ({}) failed - template output does not match expected\n\nFound {} difference(s) between expected and actual output\n\n=== DIFFERENCES ===\n{}",
            self.scope.test,
            self.scope.lang,
            differences.len(),
            differences.join("\n")
        );
        eprintln!("{}", output);
    }
}