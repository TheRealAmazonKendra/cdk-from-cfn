// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk_stack_synth::{Files, Paths};
use cdk_from_cfn::code::CodeBuffer;
use cdk_from_cfn::testing::{Paths as BasePaths, Scope};
use std::borrow::Cow;
use std::error::Error;

pub const INDENT: Cow<'static, str> = Cow::Borrowed("    ");

pub trait CdkAppCodeWriter {
    fn app_file(
        &self,
        code: &CodeBuffer,
        cdk_stack_classname: &str,
        stack_name: &str,
        include_env: bool,
    );

    fn write_app_file(
        &self,
        scope: &Scope,
        stack_name: &str,
        include_env: bool,
    ) -> Result<(), Box<dyn Error>> {
        let path = Paths::app(scope);
        let mut file = Files::create_file(&path)?;

        let code = CodeBuffer::default();
        let e2e_name = BasePaths::e2e_name(stack_name);
        self.app_file(&code, stack_name, &e2e_name, include_env);
        code.write(&mut file)
            .map_err(|e| format!("Failed to write CDK app file to {}: {}", path.display(), e))?;

        Ok(())
    }
}
