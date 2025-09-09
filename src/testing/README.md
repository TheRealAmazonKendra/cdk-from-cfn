# Testing Overview

This directory contains shared functional for the two types of synthesizer tests for `cdk-from-cfn`:

## Test Types

### 1. Synthesizer Tests (`synthesizer`)
Unit tests that directly test the internal IR Synthesizer function.

**What they test:**
- Internal `Synthesizer` function calls with CloudFormation IR input
- Verifies that the generated CDK stack files match expected stack files

**When to use:**
- Testing stack file generation in isolation
- Rapid development iteration
- Unit testing of specific CloudFormation features
- Debugging synthesis engine internals

**Run with:**
```sh
cargo test synthesizer
```

### 2. CDK Stack Synthesis Tests (`cdk-stack-synth`)
End-to-end tests that use the built `cdk-from-cfn` binary to validate the complete workflow.

**What they test:**
- Execution of the built `cdk-from-cfn` binary
- Verifies that the generated CDK stack files match expected stack files
- CDK app compilation using build script for each language `cdk synth`
- CloudFormation template equivalence (original vs synthesized) or acceptable differences in `Stack.diff`
- With `end-to-end` feature: the original template (in cases) is deployed. A change set is created for each language to validate that there are no changes on a deployed stack, verifying equivalence

**When to use:**
- Testing the complete user workflow
- Validating CloudFormation template deployability

**Run with:**
```sh
cargo test --test cdk-stack-synth
```

See [cdk-stack-synth README](../../tests/README.md) for detailed usage.

## Running All Tests

```sh
# Run both test suites
cargo test --test cdk-stack-synth --test synthesizer

# Run all tests in the workspace
cargo test
```