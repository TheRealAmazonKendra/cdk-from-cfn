// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk_stack_synth::install::fixtures::writer::{CdkAppCodeWriter, INDENT};
use cdk_from_cfn::code::{CodeBuffer, IndentOptions};

pub struct Java {}

pub const WRITER: fn() -> Box<dyn CdkAppCodeWriter> = || Box::new(Java {});

impl CdkAppCodeWriter for Java {
    fn app_file(
        &self,
        code: &CodeBuffer,
        cdk_stack_classname: &str,
        stack_name: &str,
        include_env: bool,
    ) {
        code.line("//auto-generated");
        code.line("package com.myorg;");
        code.line("import software.amazon.awscdk.App;");
        code.line("import software.amazon.awscdk.AppProps;");
        code.line("import software.amazon.awscdk.DefaultStackSynthesizer;");
        code.line("import software.amazon.awscdk.StackProps;");
        if include_env {
            code.line("import software.amazon.awscdk.Environment;");
        }
        let main_class = code.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("public class MyApp {".into()),
            trailing: Some("}".into()),
            trailing_newline: true,
        });

        let main_function = main_class.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("public static void main(final String[] args) {".into()),
            trailing: Some("}".into()),
            trailing_newline: true,
        });
        let app_constructor = main_function.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("App app = new App(AppProps.builder()".into()),
            trailing: None,
            trailing_newline: true,
        });
        let stack_synthesizer_props = app_constructor.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some(
                ".defaultStackSynthesizer(DefaultStackSynthesizer.Builder.create()".into(),
            ),
            trailing: None,
            trailing_newline: false,
        });
        stack_synthesizer_props.line(".generateBootstrapVersionRule(false)");
        stack_synthesizer_props.line(".build())");
        app_constructor.line(".build());");

        if include_env {
            let stack_props = main_function.indent_with_options(IndentOptions {
                indent: INDENT,
                leading: Some(
                    format!(
                        "new {cdk_stack_classname}(app, \"{stack_name}\", StackProps.builder()"
                    )
                    .into(),
                ),
                trailing: None,
                trailing_newline: false,
            });
            let env_props = stack_props.indent_with_options(IndentOptions {
                indent: INDENT,
                leading: Some(".env(Environment.builder()".into()),
                trailing: Some(".build())".into()),
                trailing_newline: true,
            });
            env_props.line(".region(System.getenv(\"CDK_DEFAULT_REGION\") != null ? System.getenv(\"CDK_DEFAULT_REGION\") : System.getenv(\"AWS_DEFAULT_REGION\"))");
            stack_props.line(".build());");
        } else {
            let stack_props = main_function.indent_with_options(IndentOptions {
                indent: INDENT,
                leading: Some(
                    format!(
                        "new {cdk_stack_classname}(app, \"{stack_name}\", StackProps.builder()"
                    )
                    .into(),
                ),
                trailing: None,
                trailing_newline: false,
            });
            stack_props.line(".build());");
        }
        main_function.line("app.synth();");
    }
}
