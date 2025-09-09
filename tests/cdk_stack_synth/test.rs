// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Macro for generating CDK stack synthesis test cases across multiple languages
#[macro_export]
macro_rules! cdk_stack_synth_test {
    ($name:ident, $stack_name:literal) => {
        cdk_stack_synth_test!($name, $stack_name, &[]);
    };

    ($name:ident, $stack_name:literal, $skip_cdk_synth:expr) => {
        mod $name {
            use super::*;

            static INIT: Once = Once::new();
            static INIT_RESULT: Mutex<Option<Result<&'static str, String>>> = Mutex::new(None);

            async fn generate() -> &'static str {
                let skip_list = SkipList::new($skip_cdk_synth);

                INIT.call_once_force(|_| {
                    let result = spawn(move || {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            if let Err(e) = Install::shared() {
                                return Err(format!("Install::shared failed: {}", e));
                            }

                            let langs = vec![
                                #[cfg(feature = "csharp")]
                                "csharp".to_string(),
                                #[cfg(feature = "golang")]
                                "golang".to_string(),
                                #[cfg(feature = "java")]
                                "java".to_string(),
                                #[cfg(feature = "python")]
                                "python".to_string(),
                                #[cfg(feature = "typescript")]
                                "typescript".to_string(),
                            ];

                            let region = Environment::region_for_test(stringify!($name));

                            for lang in langs {
                                let scope = Scope::new(module_path!(), &lang);
                                if let Err(e) = Install::app_file(&scope.clone(), $stack_name) {
                                    return Err(format!(
                                        "Install::app_file failed for {}: {}",
                                        lang, e
                                    ));
                                }
                                if let Err(e) = Install::boilerplate_files(&scope.clone()) {
                                    return Err(format!(
                                        "Install::boilerplate_files failed for {}: {}",
                                        lang, e
                                    ));
                                }
                                if let Err(e) = Stack::new(
                                    scope.clone(),
                                    $stack_name,
                                    <Stack as CdkFromCfnStack>::generate_stack,
                                )
                                .synth(&skip_list, region)
                                {
                                    return Err(format!("Stack::synth failed for {}: {}", lang, e));
                                }
                            }
                            #[cfg(feature = "end-to-end")]
                            {
                                // Skip if no languages will be synthesized
                                if skip_list.get_synth_languages().is_empty() {
                                    return Ok(region);
                                }
                                if let Err(e) =
                                    EndToEndTest::generate(stringify!($name), $stack_name, region)
                                        .await
                                {
                                    panic!(
                                        "End-to-end test generation failed: {}",
                                        e.message().unwrap_or("Unknown error")
                                    );
                                }
                            }
                            Ok(region)
                        })
                    })
                    .join()
                    .unwrap();
                    *INIT_RESULT.lock().unwrap_or_else(|e| e.into_inner()) = Some(result);
                });

                match INIT_RESULT
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .as_ref()
                {
                    Some(Ok(region)) => *region,
                    Some(Err(e)) => panic!("{}", e),
                    None => unreachable!(),
                }
            }

            #[cfg(feature = "csharp")]
            cdk_stack_synth_test!($name, csharp, $stack_name, $skip_cdk_synth);

            #[cfg(feature = "golang")]
            cdk_stack_synth_test!($name, golang, $stack_name, $skip_cdk_synth);

            #[cfg(feature = "java")]
            cdk_stack_synth_test!($name, java, $stack_name, $skip_cdk_synth);

            #[cfg(feature = "python")]
            cdk_stack_synth_test!($name, python, $stack_name, $skip_cdk_synth);

            #[cfg(feature = "typescript")]
            cdk_stack_synth_test!($name, typescript, $stack_name, $skip_cdk_synth);
        }
    };

    ($name: ident, $lang:ident, $stack_name:literal, $skip_cdk_synth:expr) => {
        mod $lang {
            use super::*;

            #[tokio::test]
            async fn test() {
                let name = stringify!($name);
                let lang = stringify!($lang);
                let skip_list = SkipList::new($skip_cdk_synth);
                let scope = Scope::new(module_path!(), &lang);

                let _region = super::generate().await;

                eprintln!("🚀 Starting test: cdk_stack_synth {}::{}", name, lang);

                let validator = CdkStackValidator::from_scope(&scope, $stack_name, &skip_list);
                SynthesizerTest::new(scope.clone(), name, lang)
                    .run_with_cleanup(vec![move || async move {
                        // Run validation against stored stack file
                        validator.stack_files_match_expected()?;

                        // Run validation that generated app will synth (some skipped: no further steps are taken for these tests)
                        validator.cdk_out_matches_cfn_stack_file()?;

                        // Verify that synthesized app's cdk.out template matches other languages
                        validator.synthesized_apps_match_each_other(&skip_list)?;

                        // Verify the stack will deploy and that there will be no diff between cdk stack and cfn stack
                        #[cfg(feature = "end-to-end")]
                        if let Err(e) =
                            EndToEndTest::run(&scope, $stack_name, &skip_list, _region).await
                        {
                            panic!(
                                "End-to-end test execution failed: {}",
                                e.message().unwrap_or("Unknown error")
                            );
                        }

                        Ok(())
                    }])
                    .await;

                eprint!("✅ Completed: ");
            }
        }
    };
}
