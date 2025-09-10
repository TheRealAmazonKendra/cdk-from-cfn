// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use cdk_from_cfn_testing::{Language, Scope};

#[derive(Clone, Copy)]
pub struct Issue {
    pub number: u16,
    pub description: &'static str,
}

impl Issue {
    pub const fn new(number: u16, description: &'static str) -> Self {
        Self {
            number,
            description,
        }
    }

    pub fn as_link(&self) -> String {
        let url = format!(
            "https://github.com/aws/aws-cdk-from-cfn/issues/{}",
            self.number
        );
        format!("\x1b]8;;{}\x1b\\#{}\x1b]8;;\x1b\\", url, self.number)
    }
}

#[derive(Clone)]
pub struct TestSkip {
    pub lang: String,
    pub issues: Vec<Issue>,
}

impl AsRef<str> for TestSkip {
    fn as_ref(&self) -> &str {
        &self.lang
    }
}

impl TestSkip {
    pub fn new(lang: &str, issues: Vec<Issue>) -> Self {
        Self {
            lang: lang.to_string(),
            issues,
        }
    }

    pub fn single(lang: &str, issue: Issue) -> Self {
        Self::new(lang, vec![issue])
    }

    pub fn all(issue: Issue) -> Vec<Self> {
        Language::all_languages()
            .into_iter()
            .map(|lang| Self::single(&lang, issue))
            .collect()
    }
}

pub struct SkipList {
    skip_list: Vec<TestSkip>,
}

impl SkipList {
    pub fn new(skip_list: Vec<TestSkip>) -> Self {
        Self { skip_list }
    }

    pub fn get_synth_languages(&self) -> Vec<String> {
        Language::all_languages()
            .into_iter()
            .filter(|lang| !self.skip_list.iter().any(|skip| skip.lang == *lang))
            .collect()
    }

    pub fn should_skip(&self, scope: &Scope, context: &str) -> bool {
        if let Some(skip) = self.skip_list.iter().find(|skip| skip.lang == scope.lang) {
            let issues_str = skip
                .issues
                .iter()
                .map(|issue| format!("{} ({})", issue.description, issue.as_link()))
                .collect::<Vec<_>>()
                .join(", ");
            eprintln!(
                "  ⏭️  Skipping {} for {}::{}: {}",
                context, scope.test, scope.lang, issues_str
            );
            true
        } else {
            false
        }
    }

    pub fn should_synth(&self, scope: &Scope) -> bool {
        !self.should_skip(scope, "synthesis")
    }
}

pub struct SkipSynthList;

macro_rules! skip {
    ($lang:expr, $($issue:expr),+ $(,)?) => {
        TestSkip::new($lang, vec![$($issue),+])
    };
}

impl SkipSynthList {
    const I626_GO_COMPILATION: Issue =
        Issue::new(626, "Go is an approximation at best. It does not compile");
    const I1022_CSHARP_MISSING_OUTPUTS: Issue = Issue::new(1022, "missing outputs section");
    const I1023_CSHARP_MISSING_CFN_OPTIONS: Issue = Issue::new(
        1023,
        "missing cfn options: metadata, dependencies, update policy, and deletion policy",
    );
    const I1024_JAVA_UPDATE_REPLACE: Issue = Issue::new(1024, "extra UpdateReplacePolicy key");
    const I1025_PYTHON_PARAMETER_CASING: Issue = Issue::new(
        1025,
        "parameter casing issue - camelCase instead of PascalCase",
    );
    const I1026_JAVA_MAPPING_CASING: Issue =
        Issue::new(1026, "mapping key casing - camelCase instead of PascalCase");
    const I1027_CSHARP_LAMBDA_SPACING: Issue =
        Issue::new(1027, "lambda handler spacing differences");
    const I1028_ALL_SPECIAL_CHARACTER_HANDLING: Issue = Issue::new(
        1028,
        "special characters in multiline strings aren't being handled properly",
    );
    const I1029_JAVA_NAMING_COLLISION: Issue = Issue::new(
        1029,
        "parameter sharing name with condition causes conflict",
    );

    pub fn get(test_name: &str) -> Vec<TestSkip> {
        match test_name {
            "batch" => vec![
                skip!(Language::CSHARP, Self::I1022_CSHARP_MISSING_OUTPUTS),
                skip!(Language::GOLANG, Self::I626_GO_COMPILATION),
            ],
            "cloudwatch" => vec![skip!(Language::GOLANG, Self::I626_GO_COMPILATION)],
            "config" => vec![
                skip!(
                    Language::CSHARP,
                    Self::I1022_CSHARP_MISSING_OUTPUTS,
                    Self::I1023_CSHARP_MISSING_CFN_OPTIONS
                ),
                skip!(Language::GOLANG, Self::I626_GO_COMPILATION),
            ],
            "documentdb" => vec![
                skip!(
                    Language::CSHARP,
                    Self::I1022_CSHARP_MISSING_OUTPUTS,
                    Self::I1023_CSHARP_MISSING_CFN_OPTIONS
                ),
                skip!(Language::JAVA, Self::I1024_JAVA_UPDATE_REPLACE),
                skip!(Language::PYTHON, Self::I1025_PYTHON_PARAMETER_CASING),
                skip!(Language::GOLANG, Self::I626_GO_COMPILATION),
            ],
            "ec2" => vec![skip!(Language::GOLANG, Self::I626_GO_COMPILATION)],
            "ec2_encryption" => vec![
                skip!(Language::GOLANG, Self::I626_GO_COMPILATION),
                skip!(Language::JAVA, Self::I1029_JAVA_NAMING_COLLISION),
            ],
            "ecs" => vec![skip!(Language::GOLANG, Self::I626_GO_COMPILATION)],
            "efs" => vec![
                skip!(Language::CSHARP, Self::I1022_CSHARP_MISSING_OUTPUTS),
                skip!(Language::JAVA, Self::I1026_JAVA_MAPPING_CASING),
                skip!(Language::GOLANG, Self::I626_GO_COMPILATION),
            ],
            // Due to permissions issues this template will never deploy, but it's an interesting use case for this problem
            // Once this synths we should add functionality to synth but skip deploy.
            "groundstation" => TestSkip::all(Self::I1028_ALL_SPECIAL_CHARACTER_HANDLING),
            "resource_w_json_type_properties" => {
                vec![skip!(Language::GOLANG, Self::I626_GO_COMPILATION)]
            }
            "sam_nodejs_lambda" => vec![skip!(Language::CSHARP, Self::I1027_CSHARP_LAMBDA_SPACING)],
            "sam_nodejs_lambda_arr_transform" => {
                vec![skip!(Language::CSHARP, Self::I1027_CSHARP_LAMBDA_SPACING)]
            }
            "simple" => vec![
                skip!(Language::CSHARP, Self::I1023_CSHARP_MISSING_CFN_OPTIONS),
                skip!(
                    Language::JAVA,
                    Self::I1026_JAVA_MAPPING_CASING,
                    Self::I1024_JAVA_UPDATE_REPLACE
                ),
                skip!(Language::PYTHON, Self::I1025_PYTHON_PARAMETER_CASING),
                skip!(Language::GOLANG, Self::I626_GO_COMPILATION),
            ],
            _ => vec![],
        }
    }
}
