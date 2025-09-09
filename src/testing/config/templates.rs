// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{Files, Scope};

pub enum Templates {
    Cfn {
        original_template: String,
        dependency_template: Option<String>,
    },
    Cdk {
        synthesized_template: String,
    },
}

impl Templates {
    pub fn cfn(test_name: &str) -> Self {
        let template = Files::load_case_template_from_zip(test_name)
            .unwrap_or_else(|e| panic!("Failed to load template: {}", e));
        let dependency_template = Files::load_dependency_template_from_zip(test_name).ok();

        Self::Cfn {
            original_template: template,
            dependency_template,
        }
    }

    pub fn cdk(scope: &Scope, stack_name: &str) -> Result<Self, String> {
        let synth = Files::load_actual_synthesized_template(scope, stack_name)?;
        Ok(Self::Cdk {
            synthesized_template: synth,
        })
    }

    pub fn original_template(&self) -> &str {
        match self {
            Templates::Cfn {
                original_template: template,
                ..
            } => template,
            Templates::Cdk { .. } => panic!("Called template() on Cdk variant"),
        }
    }

    pub fn dependency_template(&self) -> Option<&str> {
        match self {
            Templates::Cfn {
                dependency_template,
                ..
            } => dependency_template.as_deref(),
            Templates::Cdk { .. } => panic!("Called dependency_template() on Cdk variant"),
        }
    }

    pub fn synthesized_template(&self) -> Result<&str, String> {
        match self {
            Templates::Cfn { .. } => Err("Called synth() on Cfn variant".to_string()),
            Templates::Cdk {
                synthesized_template: synth,
            } => Ok(synth),
        }
    }
}
