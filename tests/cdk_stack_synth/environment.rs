// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(not(feature = "end-to-end"), allow(dead_code))]

use std::sync::atomic::{AtomicUsize, Ordering};

static REGION_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Environment;

impl Environment {
    const REGIONS: &'static [&'static str] = &[
        "us-east-1",
        "us-west-2",
        "eu-west-1",
        "ap-southeast-1",
        "us-east-2",
    ];

    fn default_region() -> &'static str {
        Self::REGIONS[0]
    }

    fn next_region() -> &'static str {
        let index = REGION_COUNTER.fetch_add(1, Ordering::Relaxed) % Self::REGIONS.len();
        Self::REGIONS[index]
    }

    pub fn region_for_test(test_name: &str) -> &'static str {
        match test_name {
            "ec2_encryption" => Self::default_region(),
            _ => Self::next_region(),
        }
    }
}
