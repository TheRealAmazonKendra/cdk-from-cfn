// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Macro for generating IR synthesizer test cases across multiple languages
#[macro_export]
macro_rules! ir_synthesizer_test {
    ($name:ident, $stack_name:literal) => {
        mod $name {
            use super::*;

            #[cfg(feature = "csharp")]
            ir_synthesizer_test!($name, csharp, $stack_name);

            #[cfg(feature = "golang")]
            ir_synthesizer_test!($name, golang, $stack_name);

            #[cfg(feature = "java")]
            ir_synthesizer_test!($name, java, $stack_name);

            #[cfg(feature = "python")]
            ir_synthesizer_test!($name, python, $stack_name);

            #[cfg(feature = "typescript")]
            ir_synthesizer_test!($name, typescript, $stack_name);
        }
    };

    ($name:ident, $lang:ident, $stack_name:literal) => {
        mod $lang {
            use super::*;

            #[tokio::test]
            async fn test() {
                let name = stringify!($name);
                let lang = stringify!($lang);

                let scope = Scope::new(module_path!(), lang);
                let validator = Stack::new(
                    scope.clone(),
                    $stack_name,
                    <Stack as IrStack>::generate_stack,
                )
                .validator();

                SynthesizerTest::new(scope, name, lang)
                    .run_with_cleanup(
                        (&[|| async { validator.actual_stack_files_match_expected() }]).to_vec(),
                    )
                    .await;
            }
        }
    };
}
