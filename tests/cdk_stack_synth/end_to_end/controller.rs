// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(not(feature = "end-to-end"), allow(dead_code))]

use aws_sdk_cloudformation::{
    operation::describe_change_set::DescribeChangeSetOutput,
    types::{ChangeSetType, OnStackFailure, Tag},
};

use super::AwsClient;
use crate::cdk_stack_synth::Paths;
use cdk_from_cfn::testing::Paths as BasePaths;

use aws_sdk_s3::error::ErrorMetadata;
use cdk_from_cfn::testing::{Scope, Templates};
use serde_json::{from_str, to_string, Value as JsonValue};

#[derive(Debug)]
struct FailureEvent {
    pub resource_type: String,
    pub physical_id: String,
    pub resource_status_reason: String,
}

struct BaseController {
    client: AwsClient,
    templates: Templates,
}

pub struct EndToEndTestController {
    base: BaseController,
    scope: Scope,
}

pub struct EndToEndFixtureController {
    base: BaseController,
    test: String,
}

impl BaseController {
    async fn new(test_name: &str, region: &str) -> Self {
        let templates = Templates::cfn(test_name);
        let client = AwsClient::new(region).await;
        Self { client, templates }
    }

    async fn deploy_stacks(&self, stack_name: &str, test_name: &str) -> Result<(), ErrorMetadata> {
        // Try dependency stack first (optional)
        if let Some(dep_template) = self.templates.dependency_template() {
            let dep_stack_name = Paths::dependency_name(stack_name);
            Controller::create(
                self.client.clone(),
                &dep_stack_name,
                dep_template,
                BasePaths::E2E_DEPENDENCY_TAG,
                test_name,
            )
            .await
            .ok();
        }

        // Create main stack
        let main_template = self.templates.original_template();
        let main_stack_name = BasePaths::e2e_name(stack_name);
        Controller::create(
            self.client.clone(),
            &main_stack_name,
            main_template,
            BasePaths::E2E_TAG,
            test_name,
        )
        .await
    }

    async fn cleanup_stacks(&self, stack_name: &str, test_name: &str) -> Result<(), ErrorMetadata> {
        // Cleanup main stack first
        let main_stack_name = BasePaths::e2e_name(stack_name);
        Controller::delete(
            self.client.clone(),
            &main_stack_name,
            BasePaths::E2E_TAG,
            test_name,
        )
        .await?;

        // Then dependency stack
        let dep_stack_name = Paths::dependency_name(stack_name);
        Controller::delete(
            self.client.clone(),
            &dep_stack_name,
            BasePaths::E2E_DEPENDENCY_TAG,
            test_name,
        )
        .await
    }
}

impl EndToEndFixtureController {
    pub async fn generate(test: String, region: &str) -> Self {
        let base = BaseController::new(&test, region).await;
        Self { base, test }
    }

    pub async fn await_test_stack_infrastructure_deployment(
        &self,
        stack_name: &str,
    ) -> Result<(), ErrorMetadata> {
        self.base.deploy_stacks(stack_name, &self.test).await
    }

    pub async fn await_test_stack_infrastructure_cleanup(
        &self,
        stack_name: &str,
    ) -> Result<(), ErrorMetadata> {
        self.base.cleanup_stacks(stack_name, &self.test).await
    }
}

impl EndToEndTestController {
    pub async fn new(scope: &Scope, region: &str) -> Self {
        let base = BaseController::new(&scope.test, region).await;
        Self {
            base,
            scope: scope.clone(),
        }
    }

    pub async fn check_change_sets_for_languages(
        &self,
        stack_name: &str,
        skip_list: &crate::cdk_stack_synth::SkipList,
    ) -> Result<bool, ErrorMetadata> {
        let main_stack_name = BasePaths::e2e_name(stack_name);
        let change_sets = self.base.client.list_change_sets(&main_stack_name).await?;

        Ok(skip_list.get_synth_languages().iter().all(|lang| {
            let expected_change_set_name = format!("{}-update-{}", main_stack_name, lang);
            change_sets
                .summaries()
                .iter()
                .any(|cs| cs.change_set_name().unwrap_or("") == expected_change_set_name)
        }))
    }

    pub async fn check_update_change_set(&self, stack_name: &str) -> Result<(), ErrorMetadata> {
        let main_stack_name = BasePaths::e2e_name(stack_name);
        let controller = Controller {
            stack_name: &main_stack_name,
            client: self.base.client.clone(),
            template: None,
            tag: Tag::builder()
                .key(BasePaths::E2E_TAG)
                .value(&self.scope.test)
                .build(),
            change_set_name: Some(format!("{}-update-{}", main_stack_name, self.scope.lang)),
        };
        let cdk_templates = Templates::cdk(&self.scope, stack_name).unwrap();
        let result = match controller
            .create_update_changeset(cdk_templates.synthesized_template().unwrap())
            .await
        {
            Err(e) => Err(e),
            Ok(result) => {
                if let Some(changes) = result.changes {
                    if changes.is_empty() {
                        eprintln!("  ✨ No changes detected in change set. Deployed CFN stack matches CDK stack for {}::{}", self.scope.test, self.scope.lang);
                    } else {
                        eprintln!(
                            "  ❌ Changes detected in change set for {}::{}:",
                            self.scope.test, self.scope.lang
                        );
                        for change in changes.iter() {
                            if let Some(resource_change) = change.resource_change.as_ref() {
                                let action = resource_change
                                    .action
                                    .as_ref()
                                    .map_or("Unknown", |a| a.as_str());
                                let logical_id = resource_change
                                    .logical_resource_id
                                    .as_deref()
                                    .unwrap_or("Unknown");
                                let resource_type = resource_change
                                    .resource_type
                                    .as_deref()
                                    .unwrap_or("Unknown");

                                eprintln!("    {} {} ({})", action, logical_id, resource_type);

                                if let Some(details) = resource_change.details.as_ref() {
                                    for detail in details.iter() {
                                        if let Some(target) = detail.target.as_ref() {
                                            let attr = target
                                                .attribute
                                                .as_ref()
                                                .map_or("Unknown", |a| a.as_str());
                                            let before =
                                                target.before_value.as_deref().unwrap_or("<none>");
                                            let after =
                                                target.after_value.as_deref().unwrap_or("<none>");
                                            eprintln!("      {}: {} → {}", attr, before, after);
                                        }
                                    }
                                }
                            }
                        }
                        return Err(ErrorMetadata::builder()
                            .code("ChangesDetected")
                            .message(format!("Changes detected in change set for {}::{} - deployed CFN stack differs from CDK stack", self.scope.test, self.scope.lang))
                            .build());
                    }
                }
                Ok(())
            }
        };

        result
    }
}

struct Controller<'a> {
    stack_name: &'a str,
    client: AwsClient,
    template: Option<&'a str>,
    tag: Tag,
    change_set_name: Option<String>,
}

impl<'a> Controller<'a> {
    async fn create(
        client: AwsClient,
        stack_name: &'a str,
        template: &'a str,
        tag_key: &str,
        test_name: &str,
    ) -> Result<(), ErrorMetadata> {
        let tag = Tag::builder().key(tag_key).value(test_name).build();
        let controller = Self {
            stack_name,
            client,
            template: Some(template),
            tag,
            change_set_name: Some(format!("{}-create", stack_name)),
        };
        controller.await_stack_creation().await
    }

    async fn delete(
        client: AwsClient,
        stack_name: &'a str,
        tag_key: &str,
        test_name: &str,
    ) -> Result<(), ErrorMetadata> {
        let tag = Tag::builder().key(tag_key).value(test_name).build();
        let controller = Self {
            stack_name,
            client,
            template: None,
            tag,
            change_set_name: None,
        };
        controller.await_stack_deletion().await
    }

    async fn await_stack_creation(&self) -> Result<(), ErrorMetadata> {
        let result = self.create_stack_workflow().await;
        if result.is_ok() {
            eprintln!("  🚀 {} successfully created", self.stack_name);
        }
        result
    }

    async fn create_stack_workflow(&self) -> Result<(), ErrorMetadata> {
        let output = self
            .create_change_set_with_defaults(ChangeSetType::Create, OnStackFailure::Delete)
            .await?;
        self.execute_change_set(output.change_set_id.as_ref().unwrap())
            .await
    }

    async fn create_change_set(
        &self,
        change_set_type: ChangeSetType,
        change_set_name: &str,
        template: &str,
        on_stack_failure: OnStackFailure,
    ) -> Result<DescribeChangeSetOutput, ErrorMetadata> {
        let output = match self
            .client
            .create_change_set(
                self.stack_name,
                template,
                self.tag.clone(),
                change_set_type.clone(),
                change_set_name,
                on_stack_failure.clone(),
            )
            .await
        {
            Err(e) => {
                if e.message()
                    .unwrap_or("Unknown")
                    .contains("already exists and cannot be created again")
                {
                    self.delete_change_set(change_set_name).await?;
                    return Box::pin(self.create_change_set(
                        change_set_type,
                        change_set_name,
                        template,
                        on_stack_failure,
                    ))
                    .await;
                } else {
                    return Err(self.build_client_error(e, "create change set"));
                }
            }
            Ok(result) => result,
        };

        self.client
            .wait_for_change_set_create_complete(
                output.stack_id.as_ref().unwrap(),
                output.id.as_ref().unwrap(),
            )
            .await
            .map_err(|e| self.build_client_error(e, "create change set"))
    }

    async fn execute_change_set(&self, change_set_id: &str) -> Result<(), ErrorMetadata> {
        self.client
            .execute_change_set(self.stack_name, change_set_id)
            .await
            .map_err(|e| {
                self.build_client_error(e, "execute change set. It could not be triggered")
            })?;

        self.client
            .wait_for_stack_create_complete(self.stack_name)
            .await
            .map(|_| ())
            .map_err(|e| self.build_client_error(e, "execute change set"))
    }

    async fn create_update_changeset(
        &self,
        template: &str,
    ) -> Result<DescribeChangeSetOutput, ErrorMetadata> {
        self.create_change_set(
            ChangeSetType::Update,
            self.change_set_name.as_ref().unwrap(),
            template,
            OnStackFailure::DoNothing,
        )
        .await
    }

    async fn create_change_set_with_defaults(
        &self,
        change_set_type: ChangeSetType,
        on_stack_failure: OnStackFailure,
    ) -> Result<DescribeChangeSetOutput, ErrorMetadata> {
        self.create_change_set(
            change_set_type,
            self.change_set_name.as_ref().unwrap(),
            self.template.unwrap(),
            on_stack_failure,
        )
        .await
    }

    fn build_client_error(&self, error: ErrorMetadata, action: &str) -> ErrorMetadata {
        ErrorMetadata::builder()
            .code(error.code().unwrap_or("Unknown"))
            .message(format!(
                "Failed to {action} for {}: {}",
                self.stack_name,
                error.message().unwrap_or("Unknown error")
            ))
            .build()
    }

    async fn await_stack_deletion(&self) -> Result<(), ErrorMetadata> {
        self.delete_stack_workflow().await
    }

    async fn delete_stack_workflow(&self) -> Result<(), ErrorMetadata> {
        // If the stack doesn't exist there's nothing more to do
        if self.client.describe_stacks(self.stack_name).await.is_err() {
            return Ok(());
        }

        // Check for the correct tag before deletion
        self.stack_has_e2e_tags().await?;

        // Update retention policies to Delete before attempting deletion
        if self.has_non_delete_policies(self.stack_name).await {
            self.update_retention_policies_to_delete().await?;
        }

        self.delete_stack().await?;
        self.wait_for_stack_delete_complete().await
    }

    async fn stack_has_e2e_tags(&self) -> Result<(), ErrorMetadata> {
        let output = self.client.describe_stacks(self.stack_name).await.unwrap();
        let stack_tags = output.stacks().first().unwrap().tags();
        if !stack_tags.iter().any(|stack_tag| {
            stack_tag.key() == self.tag.key() && stack_tag.value() == self.tag.value()
        }) {
            Err(ErrorMetadata::builder()
                .message(format!(
                    "{} does not have the required tag [{:?}]. The stack will not be deleted.",
                    self.stack_name, self.tag
                ))
                .build())
        } else {
            Ok(())
        }
    }

    async fn delete_stack(&self) -> Result<(), ErrorMetadata> {
        self.client
            .delete_stack(self.stack_name)
            .await
            .map(|_| ())
            .map_err(|e| self.build_client_error(e, "delete stack"))
    }

    async fn wait_for_stack_delete_complete(&self) -> Result<(), ErrorMetadata> {
        match self
            .client
            .wait_for_stack_delete_complete(self.stack_name)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.code().unwrap() == "FailureEvent" {
                    self.handle_delete_failed().await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn handle_delete_failed(&self) -> Result<(), ErrorMetadata> {
        let failure = self.get_failure_event().await?;

        if self.is_s3_bucket_not_empty_error(&failure) {
            self.empty_and_delete_bucket(&failure.physical_id).await;
            self.delete_stack().await
        } else {
            Err(self.build_client_error(
                ErrorMetadata::builder()
                    .message(failure.resource_status_reason)
                    .code("FailureEvent")
                    .build(),
                "stack deletion",
            ))
        }
    }

    fn is_s3_bucket_not_empty_error(&self, failure: &FailureEvent) -> bool {
        failure.resource_type == "AWS::S3::Bucket"
            && failure
                .resource_status_reason
                .contains("The bucket you tried to delete is not empty")
    }

    async fn get_failure_event(&self) -> Result<FailureEvent, ErrorMetadata> {
        match self.client.describe_stack_events(self.stack_name).await {
            Ok(output) => {
                let failure_events: Vec<_> = output
                    .stack_events()
                    .iter()
                    .filter(|event| {
                        event
                            .resource_status()
                            .is_some_and(|status| status.as_str().contains("FAILED"))
                    })
                    .collect();

                Ok(failure_events
                    .last()
                    .map(|event| FailureEvent {
                        resource_type: event.resource_type().unwrap_or("Unknown").to_string(),
                        physical_id: event
                            .physical_resource_id()
                            .unwrap_or("Unknown")
                            .to_string(),
                        resource_status_reason: event
                            .resource_status_reason()
                            .unwrap_or("Unknown")
                            .to_string(),
                    })
                    .expect("No failure events found"))
            }
            Err(e) => Err(e),
        }
    }

    async fn empty_and_delete_bucket(&self, bucket_name: &str) {
        loop {
            if let Ok(output) = self.client.list_objects_v2(bucket_name).await {
                for object in output.contents() {
                    if let Some(tag_key) = object.key() {
                        match self.client.delete_object(bucket_name, tag_key).await {
                            Ok(_) => {}
                            Err(_) => {
                                continue;
                            }
                        }
                    }
                }
            }

            match self.client.delete_bucket(bucket_name).await {
                Ok(_) => break,
                Err(_) => {
                    continue; // Continue retrying if bucket deletion failed
                }
            }
        }
    }

    pub async fn delete_change_set(&self, change_set_name: &str) -> Result<(), ErrorMetadata> {
        self.client
            .delete_change_set(self.stack_name, change_set_name)
            .await
            .map_err(|e| self.build_client_error(e, "delete change set"))
    }

    async fn update_retention_policies_to_delete(&self) -> Result<(), ErrorMetadata> {
        let template = self
            .client
            .get_template(self.stack_name)
            .await
            .map_err(|e| self.build_client_error(e, "get template"))?
            .template_body()
            .unwrap_or_default()
            .to_string();
        let updated_template = self.modify_template_retention_policies(&template)?;

        self.client
            .update_stack(self.stack_name, &updated_template)
            .await
            .map_err(|e| self.build_client_error(e, "update retention policies"))?;

        self.client
            .wait_for_stack_update_complete(self.stack_name)
            .await
            .map(|_| ())
            .map_err(|e| self.build_client_error(e, "wait for stack update complete"))
    }

    fn modify_template_retention_policies(&self, template: &str) -> Result<String, ErrorMetadata> {
        let mut template_json: JsonValue = self.parse_template(template)?;

        if let Some(resources) = template_json
            .get_mut("Resources")
            .and_then(|r| r.as_object_mut())
        {
            for resource in resources.values_mut() {
                if let Some(resource_obj) = resource.as_object_mut() {
                    self.update_retention_policy(resource_obj, "DeletionPolicy");
                    self.update_retention_policy(resource_obj, "UpdateReplacePolicy");
                }
            }
        }

        self.serialize_template(&template_json)
    }

    fn parse_template(&self, template: &str) -> Result<JsonValue, ErrorMetadata> {
        from_str(template).map_err(|e| {
            ErrorMetadata::builder()
                .code("InvalidTemplate")
                .message(format!("Failed to parse template: {}", e))
                .build()
        })
    }

    fn serialize_template(&self, template_json: &JsonValue) -> Result<String, ErrorMetadata> {
        to_string(template_json).map_err(|e| {
            ErrorMetadata::builder()
                .code("SerializationError")
                .message(format!("Failed to serialize template: {}", e))
                .build()
        })
    }

    fn update_retention_policy(
        &self,
        resource_obj: &mut serde_json::Map<String, JsonValue>,
        policy_key: &str,
    ) {
        if resource_obj.contains_key(policy_key) {
            resource_obj.insert(
                policy_key.to_string(),
                JsonValue::String("Delete".to_string()),
            );
        }
    }

    async fn has_non_delete_policies(&self, stack_name: &str) -> bool {
        let template = match self.client.get_template(stack_name).await {
            Ok(output) => output.template_body().unwrap_or_default().to_string(),
            Err(_) => return false,
        };

        if let Ok(template_json) = from_str::<JsonValue>(&template) {
            if let Some(resources) = template_json.get("Resources").and_then(|r| r.as_object()) {
                return resources.values().any(|resource| {
                    resource
                        .get("DeletionPolicy")
                        .and_then(|p| p.as_str())
                        .is_some_and(|p| p != "Delete")
                        || resource
                            .get("UpdateReplacePolicy")
                            .and_then(|p| p.as_str())
                            .is_some_and(|p| p != "Delete")
                });
            }
        }
        false
    }
}
