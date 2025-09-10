//! Test fixtures for CDK synthesis testing.
//!
//! This module contains a directory for each language supported by cdk-from-cfn.
//! Each language-specific directory contains the minimum necessary boilerplate
//! files for running a CDK application in that language. During runtime execution
//! of the tests, all files in a language's directory here are copied to
//! the temporary working directory for each test case.

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod writer;

use cdk_from_cfn_testing::Scope;
use writer::CdkAppCodeWriter;

#[cfg(feature = "typescript")]
pub mod typescript {
    mod writer;
    pub use writer::WRITER as typescript;
}
#[cfg(feature = "python")]
pub mod python {
    mod writer;
    pub use writer::WRITER as python;
}
#[cfg(feature = "java")]
pub mod java {
    mod writer;
    pub use writer::WRITER as java;
}
#[cfg(feature = "golang")]
pub mod golang {
    mod writer;
    pub use writer::WRITER as golang;
}
#[cfg(feature = "csharp")]
pub mod csharp {
    mod writer;
    pub use writer::WRITER as csharp;
}

#[cfg(feature = "csharp")]
pub use csharp::csharp;
#[cfg(feature = "golang")]
pub use golang::golang;
#[cfg(feature = "java")]
pub use java::java;
#[cfg(feature = "python")]
pub use python::python;
#[cfg(feature = "typescript")]
pub use typescript::typescript;

fn create_app_writer(language: &str) -> Box<dyn CdkAppCodeWriter> {
    match language {
        #[cfg(feature = "typescript")]
        "typescript" => typescript(),
        #[cfg(feature = "python")]
        "python" => python(),
        #[cfg(feature = "java")]
        "java" => java(),
        #[cfg(feature = "golang")]
        "golang" => golang(),
        #[cfg(feature = "csharp")]
        "csharp" => csharp(),
        _ => panic!("Unknown language: {}", language),
    }
}

pub struct Writer;

impl Writer {
    pub fn write_app_file(
        scope: &Scope,
        stack_name: &str,
        include_env: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let writer = create_app_writer(&scope.lang);
        writer.write_app_file(scope, stack_name, include_env)
    }
}
