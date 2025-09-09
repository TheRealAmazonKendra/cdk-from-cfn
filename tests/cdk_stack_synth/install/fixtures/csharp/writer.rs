// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk_stack_synth::install::fixtures::writer::{CdkAppCodeWriter, INDENT};
use cdk_from_cfn::code::{CodeBuffer, IndentOptions};

pub struct CSharp {}

pub const WRITER: fn() -> Box<dyn CdkAppCodeWriter> = || Box::new(CSharp {});

impl CdkAppCodeWriter for CSharp {
    fn app_file(
        &self,
        code: &CodeBuffer,
        cdk_stack_classname: &str,
        stack_name: &str,
        include_env: bool,
    ) {
        code.line("//Auto-generated");
        code.line("using Amazon.CDK;");
        code.line("sealed class Program");
        let main_class = code.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("{".into()),
            trailing: Some("}".into()),
            trailing_newline: true,
        });
        main_class.line("public static void Main(string[] args)");
        let main_function = main_class.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("{".into()),
            trailing: Some("}".into()),
            trailing_newline: true,
        });
        main_function.line("var app = new App(new AppProps");
        let app_constructor = main_function.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("{".into()),
            trailing: Some("});".into()),
            trailing_newline: true,
        });
        app_constructor.line("DefaultStackSynthesizer = new DefaultStackSynthesizer(new DefaultStackSynthesizerProps");
        let stack_synthesizer_props = app_constructor.indent_with_options(IndentOptions {
            indent: INDENT,
            leading: Some("{".into()),
            trailing: Some("}),".into()),
            trailing_newline: true,
        });
        stack_synthesizer_props.line("GenerateBootstrapVersionRule = false,");

        if include_env {
            main_function.line(format!(
                "new {cdk_stack_classname}.{cdk_stack_classname}(app, \"{stack_name}\", new {cdk_stack_classname}.{cdk_stack_classname}Props {{ Env = new Amazon.CDK.Environment {{ Region = System.Environment.GetEnvironmentVariable(\"CDK_DEFAULT_REGION\") ?? System.Environment.GetEnvironmentVariable(\"AWS_DEFAULT_REGION\") }} }});"
            ));
        } else {
            main_function.line(format!(
                "new {cdk_stack_classname}.{cdk_stack_classname}(app, \"{stack_name}\");"
            ));
        }
        main_function.line("app.Synth();");
    }
}
