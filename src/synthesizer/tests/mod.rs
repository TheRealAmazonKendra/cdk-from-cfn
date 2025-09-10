// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
use crate::ir_synthesizer_test;

mod stack;
mod test;
#[cfg(test)]
use cdk_from_cfn_testing::{Scope, Stack, SynthesizerTest};
use cdk_from_cfn_macros::generate_ir_tests;
use stack::IrStack;

generate_ir_tests!();
