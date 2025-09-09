// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk_stack_synth::install::fixtures::writer::{CdkAppCodeWriter, INDENT};
use cdk_from_cfn::code::{CodeBuffer, IndentOptions};

pub struct Typescript;

pub const WRITER: fn() -> Box<dyn CdkAppCodeWriter> = || Box::new(Typescript {});

impl CdkAppCodeWriter for Typescript {
    fn app_file(
        &self,
        code: &CodeBuffer,
        cdk_stack_classname: &str,
        stack_name: &str,
        include_env: bool,
    ) {
        code.line("// auto-generated! a human should update this!");
        code.line("import * as cdk from \"aws-cdk-lib\";");
        code.line(format!(
            "import {{ {cdk_stack_classname} }} from \"./{cdk_stack_classname}\";"
        ));
        let app = code.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("const app = new cdk.App({".into()),
            trailing: Some("});".into()),
            trailing_newline: true,
        });
        let app_props = app.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("defaultStackSynthesizer: new cdk.DefaultStackSynthesizer({".into()),
            trailing: Some("}),".into()),
            trailing_newline: true,
        });
        app_props.line("generateBootstrapVersionRule: false,");
        if include_env {
            code.line(format!("new {cdk_stack_classname}(app, \"{stack_name}\", {{ env: {{ region: process.env.CDK_DEFAULT_REGION || process.env.AWS_DEFAULT_REGION }} }});"));
        } else {
            code.line(format!("new {cdk_stack_classname}(app, \"{stack_name}\");"));
        }
        code.line("app.synth();");
    }
}
