// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk_stack_synth::install::fixtures::writer::{CdkAppCodeWriter, INDENT};
use cdk_from_cfn::code::{CodeBuffer, IndentOptions};

pub struct Golang {}

pub const WRITER: fn() -> Box<dyn CdkAppCodeWriter> = || Box::new(Golang {});

impl CdkAppCodeWriter for Golang {
    fn app_file(
        &self,
        code: &CodeBuffer,
        cdk_stack_classname: &str,
        stack_name: &str,
        include_env: bool,
    ) {
        code.line("// auto-generated");
        code.line("package main");
        let imports = code.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("import (".into()),
            trailing: Some(")".into()),
            trailing_newline: true,
        });
        if include_env {
            imports.line("\"os\"");
        }
        imports.line("\"github.com/aws/aws-cdk-go/awscdk/v2\"");
        imports.line("\"github.com/aws/jsii-runtime-go\"");

        let main_function = code.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("func main() {".into()),
            trailing: Some("}".into()),
            trailing_newline: true,
        });
        main_function.line("defer jsii.Close()");
        let app_constructor = main_function.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("app := awscdk.NewApp(&awscdk.AppProps{".into()),
            trailing: Some("})".into()),
            trailing_newline: true,
        });
        let stack_synthesizer_props = app_constructor.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("DefaultStackSynthesizer: awscdk.NewDefaultStackSynthesizer(&awscdk.DefaultStackSynthesizerProps{".into()),
            trailing: Some("}),".into()), 
            trailing_newline: true,
        });
        stack_synthesizer_props.line("GenerateBootstrapVersionRule: jsii.Bool(false),");

        if include_env {
            main_function.line("region := os.Getenv(\"CDK_DEFAULT_REGION\")");
            main_function.line("if region == \"\" {");
            main_function.line("    region = os.Getenv(\"AWS_DEFAULT_REGION\")");
            main_function.line("}");
            main_function.line(format!("New{cdk_stack_classname}(app, \"{stack_name}\", &awscdk.StackProps{{ Env: &awscdk.Environment{{ Region: jsii.String(region) }} }})"));
        } else {
            main_function.line(format!(
                "New{cdk_stack_classname}(app, \"{stack_name}\", nil)"
            ));
        }
        main_function.line("app.Synth(nil)");
    }
}
