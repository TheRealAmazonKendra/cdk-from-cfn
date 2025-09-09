// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{Files, Language, Scope};

#[derive(Clone)]
pub struct Stacks {
    pub expected: String,
    pub actual: String,
}

impl Stacks {
    pub fn new(scope: Scope, content: &str, stack_name: &str) -> Self {
        let processed_content = Language::post_process_output(&scope.lang, content.to_string());

        let expected_content = if cfg!(feature = "update-snapshots") {
            // Write the new/updated test case stack to expected on disk
            Files::write_expected_stack(&scope, stack_name, &processed_content)
                .unwrap_or_else(|e| eprintln!("Failed to write expected stack: {}", e));
            eprintln!(
                "  🪄 Created expected file for {}::{}",
                scope.test, scope.lang
            );

            // Use the new/updated test stack as expected for this test run
            Files::load_expected_stack(&scope.test, &scope.lang).unwrap_or_else(|e| {
                panic!(
                    "Failed to read newly created expected stack for {}::{}: {e}",
                    &scope.test, &scope.lang
                )
            })
        } else {
            Files::load_expected_stack_from_zip(&scope.test, &scope.lang)
                .unwrap_or_else(|e| panic!("Failed to load expected stack: {}", e))
        };

        Files::write_actual_stack(&scope, &processed_content)
            .unwrap_or_else(|e| panic!("Failed to write actual stack: {}", e));

        Self {
            expected: expected_content,
            actual: Files::load_actual_stack(&scope)
                .unwrap_or_else(|e| panic!("Failed to load actual stack: {}", e)),
        }
    }
    pub fn from(scope: &Scope) -> Self {
        let expected = if cfg!(feature = "update-snapshots") {
            // Load the new/updated stack from disk instead of the zip as it will not be there for current test run
            Files::load_expected_stack(&scope.test, &scope.lang).unwrap_or_else(|e| {
                panic!(
                    "Failed to read newly created expected stack for {}::{}: {e}",
                    &scope.test, &scope.lang
                )
            })
        } else {
            Files::load_expected_stack_from_zip(&scope.test, &scope.lang)
                .unwrap_or_else(|e| panic!("Failed to load expected stack: {}", e))
        };
        Self {
            expected,
            actual: Files::load_actual_stack(scope)
                .unwrap_or_else(|e| panic!("Failed to load actual stack: {}", e)),
        }
    }
}
