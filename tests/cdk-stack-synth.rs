// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod cdk_stack_synth;

#[cfg(feature = "end-to-end")]
use cdk_stack_synth::EndToEndTest;

use cdk_from_cfn::testing::{Scope, Stack, SynthesizerTest};
use cdk_from_cfn_macros::generate_cdk_tests;
use cdk_stack_synth::{
    CdkFromCfnStack, CdkStackValidator, Environment, Install, SkipList, SkipSynthList,
};

use std::{
    sync::{Mutex, Once},
    thread::spawn,
};

use tokio::runtime::Runtime;

generate_cdk_tests!();
