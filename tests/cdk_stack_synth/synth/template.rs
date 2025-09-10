// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use cdk_from_cfn_testing::{Files, Paths as BasePaths};
use cdk_from_cfn_testing::{Files as BaseFiles, Scope};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// Handles comparison of JSON CloudFormation templates with semantic understanding
/// Ignores formatting differences and provides detailed diff analysis
pub struct Template;

impl Template {
    /// Check if a key should be ignored in comparisons
    fn is_ignored_key(key: &str) -> bool {
        key == "AWSTemplateFormatVersion" || key == "Description"
    }

    /// Build path for nested JSON traversal
    fn build_path(base: &str, key: &str) -> String {
        if base.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", base, key)
        }
    }

    /// Pretty print JSON value with fallback
    fn pretty_print_json(value: &Value) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| format!("{:?}", value))
    }

    /// Compare two JSON templates semantically, ignoring order and formatting
    fn compare_templates(
        expected_content: &str,
        actual_content: &str,
        test_name: &str,
    ) -> Result<bool, String> {
        let expected_json = Self::parse_json_with_context(expected_content, "expected")?;
        let actual_json = Self::parse_json_with_context(actual_content, "actual")?;

        let normalized_match = Self::compare_json_templates(&expected_json, &actual_json);
        let current_diff = Self::generate_detailed_diff(&expected_json, &actual_json);

        // Update diff file when update_snapshots is enabled
        if cfg!(feature = "update-snapshots") {
            let acceptable_diff_path = BasePaths::case_path(test_name, "Stack.diff");
            if normalized_match {
                // Delete diff file if it exists since there are no differences
                Files::remove_file(&acceptable_diff_path).ok();
            } else {
                Files::write_diff(&acceptable_diff_path, &current_diff).ok();
            }
            return Ok(true);
        }

        if normalized_match {
            return Ok(true);
        }

        // Check if existing diff matches current diff
        Self::check_diff_acceptable(test_name, &current_diff)
    }

    fn check_diff_acceptable(test_name: &str, current_diff: &str) -> Result<bool, String> {
        if let Ok(acceptable_diff) = Files::load_acceptable_diff(test_name) {
            let matches = acceptable_diff.trim() == current_diff.trim();
            if !matches {
                return Err(format!(
                    "Template differences are not acceptable:\n{}",
                    current_diff
                ));
            }
            Ok(true)
        } else {
            Err(format!(
                "Template differences found (no acceptable diff file):\n{}",
                current_diff
            ))
        }
    }

    /// Normalize JSON by sorting objects and handling arrays consistently
    fn normalize_json(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut sorted_map = BTreeMap::new();
                for (k, v) in map {
                    if !Self::is_ignored_key(k) {
                        sorted_map.insert(k.clone(), Self::normalize_json(v));
                    }
                }
                Value::Object(sorted_map.into_iter().collect())
            }
            Value::Array(arr) => Value::Array(arr.iter().map(Self::normalize_json).collect()),
            _ => value.clone(),
        }
    }

    /// Compare two JSON values semantically, returning true if they match
    pub fn compare_json_templates(expected: &Value, actual: &Value) -> bool {
        Self::normalize_json(expected) == Self::normalize_json(actual)
    }

    /// Parse JSON string with error context
    fn parse_json_with_context(json_str: &str, context: &str) -> Result<Value, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse {} template: {}", context, e))
    }

    /// Compare multiple templates and return error if any don't match
    pub fn compare_multiple_templates(templates: &HashMap<String, String>) -> Result<(), String> {
        let template_vec: Vec<_> = templates.iter().collect();

        let (first_lang, first_template) = match template_vec.first() {
            Some(first) => first,
            None => return Ok(()),
        };

        let first_json = Self::parse_json_with_context(first_template, first_lang)?;

        for (lang, template) in template_vec.iter().skip(1) {
            let current_json = Self::parse_json_with_context(template, lang)?;

            if !Self::compare_json_templates(&first_json, &current_json) {
                let diff = Self::generate_detailed_diff(&first_json, &current_json);
                return Err(format!(
                    "  ❌ Synthesized CDK apps do not match between {} and {}:\n{}",
                    first_lang, lang, diff
                ));
            }
        }

        Ok(())
    }

    /// Generate detailed diff showing specific differences
    pub fn generate_detailed_diff(expected: &Value, actual: &Value) -> String {
        let mut diff_lines = Vec::new();
        Self::find_differences(expected, actual, "", &mut diff_lines);

        if diff_lines.is_empty() {
            "No differences found".to_string()
        } else {
            format!("Differences found:\n{}", diff_lines.join("\n\n"))
        }
    }

    /// Recursively find differences between JSON values
    fn find_differences(
        expected: &Value,
        actual: &Value,
        path: &str,
        diff_lines: &mut Vec<String>,
    ) {
        match (expected, actual) {
            (Value::Object(exp_map), Value::Object(act_map)) => {
                for (key, exp_val) in exp_map {
                    let current_path = Self::build_path(path, key);
                    match act_map.get(key) {
                        Some(act_val) => {
                            Self::find_differences(exp_val, act_val, &current_path, diff_lines)
                        }
                        None => {
                            if !Self::is_ignored_key(key) {
                                Self::add_missing_key_diff(&current_path, exp_val, diff_lines);
                            }
                        }
                    }
                }
                for key in act_map.keys() {
                    if !exp_map.contains_key(key) {
                        let current_path = Self::build_path(path, key);
                        Self::add_extra_key_diff(
                            &current_path,
                            act_map.get(key).unwrap(),
                            diff_lines,
                        );
                    }
                }
            }
            (Value::Array(exp_arr), Value::Array(act_arr)) => {
                if exp_arr.len() != act_arr.len() {
                    diff_lines.push(format!(
                        "Array length mismatch at {}: expected {}, got {}",
                        path,
                        exp_arr.len(),
                        act_arr.len()
                    ));
                }
                for (i, (exp_item, act_item)) in exp_arr.iter().zip(act_arr.iter()).enumerate() {
                    let current_path = format!("{}[{}]", path, i);
                    Self::find_differences(exp_item, act_item, &current_path, diff_lines);
                }
            }
            (exp, act) if exp != act => {
                Self::add_value_mismatch_diff(path, exp, act, diff_lines);
            }
            _ => {}
        }
    }

    /// Add indentation to each line of text
    fn indent(text: &str, spaces: usize) -> String {
        let indent_str = " ".repeat(spaces);
        text.lines()
            .map(|line| format!("{}{}", indent_str, line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Add missing key diff to diff_lines
    fn add_missing_key_diff(path: &str, value: &Value, diff_lines: &mut Vec<String>) {
        let pretty = Self::pretty_print_json(value);
        diff_lines.push(format!(
            "- Missing key: {}\n  Expected:\n{}",
            path,
            Self::indent(&pretty, 4)
        ));
    }

    /// Add extra key diff to diff_lines
    fn add_extra_key_diff(path: &str, value: &Value, diff_lines: &mut Vec<String>) {
        let pretty = Self::pretty_print_json(value);
        diff_lines.push(format!(
            "+ Extra key: {}\n  Actual:\n{}",
            path,
            Self::indent(&pretty, 4)
        ));
    }

    /// Add value mismatch diff to diff_lines
    fn add_value_mismatch_diff(
        path: &str,
        expected: &Value,
        actual: &Value,
        diff_lines: &mut Vec<String>,
    ) {
        let exp_pretty = Self::pretty_print_json(expected);
        let act_pretty = Self::pretty_print_json(actual);
        diff_lines.push(format!(
            "Value mismatch at {}:\n  Expected:\n{}\n  Actual:\n{}",
            path,
            Self::indent(&exp_pretty, 4),
            Self::indent(&act_pretty, 4)
        ));
    }

    /// Validate that template templates match expected templates
    pub fn validate(scope: &Scope, stack_name: &str) -> Result<(), String> {
        let expected_content = BaseFiles::load_case_template_from_zip(&scope.test)?;
        let actual_content = BaseFiles::load_actual_synthesized_template(scope, stack_name)?;

        match Self::compare_templates(&expected_content, &actual_content, &scope.test) {
            Ok(true) => {
                eprintln!(
                    "  ✨ Synthesized App {}::{} passed (templates match exactly)",
                    scope.test, scope.lang
                );
                Ok(())
            }
            Ok(false) => {
                eprintln!(
                    "  ✨ Synthesized App {}::{} passed (all differences are acceptable)",
                    scope.test, scope.lang
                );
                Ok(())
            }
            Err(e) => Err(format!(
                "Template comparison failed for {}::{}: {}",
                scope.test, scope.lang, e
            )),
        }
    }
}
