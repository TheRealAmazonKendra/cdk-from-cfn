// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{Files, Scope};
use std::future::Future;

/// Generic test structure that is extensible for both sets of synthesizer tests
/// We've overloaded the word synthesizer in cdk-from-cfn. In this context it's referring
/// to the IR Synthesizer, not the CDK synth process. We should consider changing the name
/// of that module for better clarity.
pub struct SynthesizerTest<'a> {
    scope: Scope,
    test_name: &'a str,
    lang: &'a str,
}

impl<'a> SynthesizerTest<'a> {
    pub fn new(scope: Scope, test_name: &'a str, lang: &'a str) -> Self {
        Self {
            scope,
            test_name,
            lang,
        }
    }

    pub async fn run_with_cleanup<F, Fut>(self, functions: Vec<F>)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let mut error = None;
        for function in functions {
            if let Err(e) = function().await {
                error = Some(e);
                break;
            }
        }

        if !cfg!(feature = "skip-clean") {
            if let Err(e) = Files::cleanup_test(&self.scope) {
                eprintln!("Warning: Failed to cleanup test files: {}", e);
            }
        }

        match error {
            Some(e) => panic!("{e}"),
            None => eprintln!("  ✨ Test {}::{} passed", self.test_name, self.lang),
        }
    }
}
