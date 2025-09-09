// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(not(feature = "end-to-end"), allow(dead_code))]

use aws_config::{BehaviorVersion, Region};
use aws_sdk_cloudformation::client::Waiters;
use aws_sdk_cloudformation::error::SdkError as CfnSdkError;
use aws_sdk_cloudformation::types::StackStatus;
use aws_sdk_cloudformation::{
    operation::{
        create_change_set::CreateChangeSetOutput, delete_stack::DeleteStackOutput,
        describe_change_set::DescribeChangeSetOutput,
        describe_stack_events::DescribeStackEventsOutput, describe_stacks::DescribeStacksOutput,
        execute_change_set::ExecuteChangeSetOutput, get_template::GetTemplateOutput,
        list_change_sets::ListChangeSetsOutput, update_stack::UpdateStackOutput,
    },
    types::{Capability, ChangeSetType, OnStackFailure, Tag},
    Client as CloudFormationClient,
};
use aws_sdk_s3::{
    error::SdkError as S3SdkError,
    operation::{
        delete_bucket::DeleteBucketOutput, delete_object::DeleteObjectOutput,
        list_objects_v2::ListObjectsV2Output,
    },
    Client as S3Client,
};
use aws_smithy_runtime_api::client::result::CreateUnhandledError;
use aws_smithy_runtime_api::client::waiters::{error::WaiterError, FinalPoll};
use aws_smithy_types::error::metadata::{ErrorMetadata, ProvideErrorMetadata};
use std::{error::Error, time::Duration};

fn extract_error_metadata<E, T>(result: Result<E, CfnSdkError<T>>) -> Result<E, ErrorMetadata>
where
    T: Error + Send + Sync + CreateUnhandledError + ProvideErrorMetadata + 'static,
{
    result.map_err(|e| e.into_service_error().meta().clone())
}

fn extract_s3_error_metadata<E, T>(result: Result<E, S3SdkError<T>>) -> Result<E, ErrorMetadata>
where
    T: Error + Send + Sync + CreateUnhandledError + ProvideErrorMetadata + 'static,
{
    result.map_err(|e| e.into_service_error().meta().clone())
}

fn extract_waiter_result<T: Clone + std::fmt::Debug, E>(
    result: Result<FinalPoll<T, CfnSdkError<E>>, WaiterError<T, E>>,
) -> Result<T, ErrorMetadata>
where
    E: Error + Send + Sync + CreateUnhandledError + ProvideErrorMetadata + 'static,
{
    match result {
        Ok(final_poll) => match final_poll.into_result() {
            Ok(output) => Ok(output),
            Err(e) => Err(e.into_service_error().meta().clone()),
        },
        Err(waiter_error) => match waiter_error {
            WaiterError::ExceededMaxWait(poll) => {
                Err(ErrorMetadata::builder().code("ExceededMaxWait").message(format!("Waiter exceeded maximum wait time. Time Elapsed: [{:?}] Max Wait Time: [{:?}] Poll Count: [{}]", poll.elapsed(), poll.max_wait(), poll.poll_count())).build())
            }
            WaiterError::FailureState(state) => {
                match state.into_final_poll().into_result() {
                    Ok(output) => Ok(output),
                    Err(e) => Err(ErrorMetadata::builder().code(e.code().unwrap()).message(e.message().unwrap()).build()),
                }

            }
            WaiterError::OperationFailed(operation) => {
                let error = operation.into_error().into_service_error().meta().clone();
                Err(error)
            }
            WaiterError::ConstructionFailure(error) => Err(ErrorMetadata::builder()
                .message(format!("Waiter construction failure: {:?}", error))
                .build()),
            _ => Err(ErrorMetadata::builder().code("Unknown").message("Some unexpected error occurred while waiting for a result").build()),
        },
    }
}

#[derive(Clone)]
pub struct AwsClient {
    cloudformation: CloudFormationClient,
    s3: S3Client,
}

impl AwsClient {
    pub async fn new(region: &str) -> Self {
        let config = tokio::time::timeout(
            Duration::from_secs(10),
            aws_config::load_defaults(BehaviorVersion::latest()),
        )
        .await
        .unwrap_or_else(|_| {
            panic!("❌ AWS config loading timed out after 10 seconds");
        })
        .into_builder()
        .region(Region::new(region.to_string()))
        .build();

        Self {
            cloudformation: CloudFormationClient::new(&config),
            s3: S3Client::new(&config),
        }
    }

    pub async fn create_change_set(
        &self,
        stack_name: &str,
        template_body: &str,
        tag: Tag,
        change_set_type: ChangeSetType,
        change_set_name: &str,
        on_stack_failure: OnStackFailure,
    ) -> Result<CreateChangeSetOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .create_change_set()
                .change_set_name(change_set_name)
                .stack_name(stack_name)
                .template_body(template_body)
                .change_set_type(change_set_type)
                .capabilities(Capability::CapabilityIam)
                .capabilities(Capability::CapabilityNamedIam)
                .capabilities(Capability::CapabilityAutoExpand)
                .tags(tag)
                .import_existing_resources(true)
                .on_stack_failure(on_stack_failure)
                .send()
                .await,
        )
    }

    pub async fn wait_for_change_set_create_complete(
        &self,
        stack_name: &str,
        change_set_name: &str,
    ) -> Result<DescribeChangeSetOutput, ErrorMetadata> {
        extract_waiter_result(
            self.cloudformation
                .wait_until_change_set_create_complete()
                .stack_name(stack_name)
                .change_set_name(change_set_name)
                .include_property_values(true)
                .wait(Duration::from_secs(300))
                .await,
        )
    }

    pub async fn wait_for_stack_create_complete(
        &self,
        stack_name: &str,
    ) -> Result<DescribeStacksOutput, ErrorMetadata> {
        extract_waiter_result(
            self.cloudformation
                .wait_until_stack_create_complete()
                .stack_name(stack_name)
                .wait(Duration::from_secs(1800))
                .await,
        )
    }

    pub async fn wait_for_stack_update_complete(
        &self,
        stack_name: &str,
    ) -> Result<DescribeStacksOutput, ErrorMetadata> {
        extract_waiter_result(
            self.cloudformation
                .wait_until_stack_update_complete()
                .stack_name(stack_name)
                .wait(Duration::from_secs(1800))
                .await,
        )
    }

    pub async fn wait_for_stack_delete_complete(
        &self,
        stack_name: &str,
    ) -> Result<(), ErrorMetadata> {
        match extract_waiter_result(
            self.cloudformation
                .wait_until_stack_delete_complete()
                .stack_name(stack_name)
                .wait(Duration::from_secs(1800))
                .await,
        ) {
            Err(e) => {
                if e.message()
                    .unwrap_or("Unknown error")
                    .contains("does not exist")
                {
                    Ok(())
                } else {
                    Err(e)
                }
            }
            Ok(output) => {
                let stacks = output.stacks.clone().unwrap();
                let stack = stacks.first().unwrap();
                if stack.stack_status.clone().unwrap() != StackStatus::DeleteComplete {
                    let failure = stack.stack_status_reason.clone().unwrap();
                    Err(ErrorMetadata::builder()
                        .message(failure)
                        .code("FailureEvent")
                        .build())
                } else {
                    Ok(())
                }
            }
        }
    }

    pub async fn execute_change_set(
        &self,
        stack_name: &str,
        change_set_name: &str,
    ) -> Result<ExecuteChangeSetOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .execute_change_set()
                .change_set_name(change_set_name)
                .retain_except_on_create(true)
                .stack_name(stack_name)
                .send()
                .await,
        )
    }

    pub async fn describe_stacks(
        &self,
        stack_name: &str,
    ) -> Result<DescribeStacksOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .describe_stacks()
                .stack_name(stack_name)
                .send()
                .await,
        )
    }

    pub async fn delete_stack(&self, stack_name: &str) -> Result<DeleteStackOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .delete_stack()
                .stack_name(stack_name)
                .send()
                .await,
        )
    }

    pub async fn get_template(&self, stack_name: &str) -> Result<GetTemplateOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .get_template()
                .stack_name(stack_name)
                .send()
                .await,
        )
    }

    pub async fn update_stack(
        &self,
        stack_name: &str,
        template_body: &str,
    ) -> Result<UpdateStackOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .update_stack()
                .stack_name(stack_name)
                .template_body(template_body)
                .capabilities(Capability::CapabilityIam)
                .capabilities(Capability::CapabilityNamedIam)
                .capabilities(Capability::CapabilityAutoExpand)
                .send()
                .await,
        )
    }

    pub async fn describe_stack_events(
        &self,
        stack_name: &str,
    ) -> Result<DescribeStackEventsOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .describe_stack_events()
                .stack_name(stack_name)
                .send()
                .await,
        )
    }

    pub async fn list_objects_v2(
        &self,
        bucket_name: &str,
    ) -> Result<ListObjectsV2Output, ErrorMetadata> {
        extract_s3_error_metadata(self.s3.list_objects_v2().bucket(bucket_name).send().await)
    }

    pub async fn delete_object(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> Result<DeleteObjectOutput, ErrorMetadata> {
        extract_s3_error_metadata(
            self.s3
                .delete_object()
                .bucket(bucket_name)
                .key(key)
                .send()
                .await,
        )
    }

    pub async fn delete_bucket(
        &self,
        bucket_name: &str,
    ) -> Result<DeleteBucketOutput, ErrorMetadata> {
        extract_s3_error_metadata(self.s3.delete_bucket().bucket(bucket_name).send().await)
    }

    pub async fn delete_change_set(
        &self,
        stack_name: &str,
        change_set_name: &str,
    ) -> Result<(), ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .delete_change_set()
                .stack_name(stack_name)
                .change_set_name(change_set_name)
                .send()
                .await,
        )
        .map(|_| ())
    }

    pub async fn list_change_sets(
        &self,
        stack_name: &str,
    ) -> Result<ListChangeSetsOutput, ErrorMetadata> {
        extract_error_metadata(
            self.cloudformation
                .list_change_sets()
                .stack_name(stack_name)
                .send()
                .await,
        )
    }
}
