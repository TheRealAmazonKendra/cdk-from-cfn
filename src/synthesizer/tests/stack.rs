// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cdk::Schema;
use crate::ir::CloudformationProgramIr;
use crate::testing::{Language, Stack};
use crate::CloudformationParseTree;

pub trait IrStack {
    fn generate_stack(template: &str, lang: &str, stack_name: &str) -> Vec<u8>;
}

impl IrStack for Stack {
    fn generate_stack(template: &str, lang: &str, stack_name: &str) -> Vec<u8> {
        let cfn: CloudformationParseTree = serde_json::from_str(template).unwrap();
        let ir = CloudformationProgramIr::from(cfn, Schema::builtin()).unwrap();

        let mut output = Vec::new();
        let ir_lang = Language::lang_arg(lang);
        ir.synthesize(ir_lang, &mut output, stack_name).unwrap();

        output
    }
}
