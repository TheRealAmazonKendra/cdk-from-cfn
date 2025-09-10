# cdk-from-cfn-testing

Testing utilities for `cdk-from-cfn`. This crate provides shared testing infrastructure used across both unit tests and end-to-end tests.

## Features

- **Stack Testing**: Compare generated CDK stack files against expected outputs
- **File Management**: Handle test file creation, cleanup, and organization
- **Language Support**: Multi-language testing utilities for TypeScript, Python, Java, Go, and C#
- **Template Handling**: Load and process CloudFormation templates from test cases
- **Scope Management**: Organize tests by module, test name, and language

## Usage

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
cdk-from-cfn-testing = { path = "cdk-from-cfn-testing" }
```

## Key Components

- `Stack` and `StackValidator`: Compare generated vs expected stack files
- `SynthesizerTest`: Generic test runner with cleanup
- `Files`: File I/O operations for test artifacts
- `Paths`: Path resolution for test directories and files
- `Templates`: Load CloudFormation templates from test cases
- `Scope`: Test organization and naming

## Features

- `update-snapshots`: Update expected test outputs
- `skip-clean`: Skip cleanup of test files for debugging
- `end-to-end`: Enable end-to-end testing features

This crate is not published to crates.io and is intended for internal use only.