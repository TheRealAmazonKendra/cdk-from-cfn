// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro::TokenStream;
use quote::quote;

const TEST_DEFINITIONS: &[(&str, &str)] = &[
    ("batch", "BatchStack"),
    ("bucket", "BucketStack"),
    ("cloudwatch", "CloudwatchStack"),
    ("config", "ConfigStack"),
    ("documentdb", "DocumentDbStack"),
    ("ec2", "Ec2Stack"),
    ("ec2_encryption", "Ec2EncryptionStack"),
    ("ecs", "EcsStack"),
    ("efs", "EfsStack"),
    ("groundstation", "GroundStationStack"),
    ("resource_w_json_type_properties", "JsonPropsStack"),
    ("sam_nodejs_lambda", "SAMNodeJSLambdaStack"),
    ("sam_nodejs_lambda_arr_transform", "SAMNodeJSLambdaArrStack"),
    ("simple", "SimpleStack"),
    ("vpc", "VpcStack"),
];

#[proc_macro]
pub fn generate_cdk_tests(_input: TokenStream) -> TokenStream {
    let tests = TEST_DEFINITIONS.iter().map(|(test_name, stack_name)| {
        let test_ident = syn::Ident::new(test_name, proc_macro2::Span::call_site());
        quote! {
            cdk_stack_synth_test!(#test_ident, #stack_name, SkipSynthList::get(#test_name));
        }
    });

    let expanded = quote! {
        #(#tests)*
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn generate_ir_tests(_input: TokenStream) -> TokenStream {
    let tests = TEST_DEFINITIONS.iter().map(|(test_name, stack_name)| {
        let test_ident = syn::Ident::new(test_name, proc_macro2::Span::call_site());
        quote! {
            ir_synthesizer_test!(#test_ident, #stack_name);
        }
    });

    let expanded = quote! {
        #(#tests)*
    };

    TokenStream::from(expanded)
}
