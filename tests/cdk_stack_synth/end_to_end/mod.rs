#![cfg_attr(not(feature = "end-to-end"), allow(dead_code))]

use aws_sdk_cloudformation::error::ErrorMetadata;
use futures::{FutureExt, TryFutureExt};

use crate::cdk_stack_synth::{end_to_end::controller::EndToEndFixtureController, SkipList};
use cdk_from_cfn_testing::Scope;
use std::panic::{resume_unwind, AssertUnwindSafe};

mod client;
mod controller;

use client::AwsClient;
use controller::EndToEndTestController;

pub struct EndToEndTest;

impl EndToEndTest {
    pub async fn generate(test: &str, stack_name: &str, region: &str) -> Result<(), ErrorMetadata> {
        eprintln!("  🌩️  Starting deployment for {test} for {stack_name} in {region}");
        let controller = EndToEndFixtureController::generate(test.to_string(), region).await;

        // Initial cleanup - log errors but continue
        Self::log_error_and_continue(
            controller
                .await_test_stack_infrastructure_cleanup(stack_name)
                .await,
            "Initial cleanup failed, continuing",
        );

        // Deploy - this should fail if credentials are missing
        let deploy_result = AssertUnwindSafe(
            controller
                .await_test_stack_infrastructure_deployment(stack_name)
                .map_err(|e| {
                    panic!(
                        "❌ Stack deployment failed for {} due to error code: {}\n\t{}",
                        test,
                        e.code().unwrap_or("Unknown"),
                        e.message().unwrap_or("Unknown")
                    )
                }),
        )
        .catch_unwind()
        .await;

        // Propagate deployment failures (including credential issues)
        match deploy_result {
            Ok(result) => result,
            Err(panic) => resume_unwind(panic),
        }
    }

    pub async fn run(
        scope: &Scope,
        stack_name: &str,
        skip_list: &SkipList,
        region: &str,
    ) -> Result<(), ErrorMetadata> {
        // Skip if language is in skip list
        if skip_list.should_skip(scope, "end-to-end test") {
            return Ok(());
        }

        let controller = EndToEndTestController::new(scope, region).await;
        controller.check_update_change_set(stack_name).await?;
        let _all_change_sets_found = controller
            .check_change_sets_for_languages(stack_name, skip_list)
            .await?;

        #[cfg(not(feature = "skip-clean"))]
        {
            if _all_change_sets_found {
                let test = scope.test.clone();
                eprintln!("  🌩️  Starting cleanup for {test} for {stack_name} in {region}");
                let fixture_controller = EndToEndFixtureController::generate(test, region).await;
                Self::log_error_and_continue(
                    fixture_controller
                        .await_test_stack_infrastructure_cleanup(stack_name)
                        .await,
                    "Infrastructure cleanup failed",
                );
            }
        }

        Ok(())
    }

    fn log_error_and_continue(result: Result<(), ErrorMetadata>, context: &str) {
        if let Err(e) = result {
            eprintln!(
                "  ⚠️  {}: {}",
                context,
                e.message().unwrap_or("Unknown error")
            );
        }
    }
}
