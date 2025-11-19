/******************************************************************************
* (c) 2005-2019 Copyright, Real-Time Innovations.  All rights reserved.       *
* No duplications, whole or partial, manual or electronic, may be made        *
* without express written permission.  Any such copies, or revisions thereof, *
* must display this notice unaltered.                                         *
* This code contains trade secrets of Real-Time Innovations, Inc.             *
******************************************************************************/

#[macro_use]
extern crate assert_matches;

mod test_utils;

use rtiddsconnector::{self, Connector, SelectedValue};
use std::path::Path;
use test_utils::TestContextBuilder;

/// Helper function to check if path exists
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[test]
fn test_connector_instantiation() {
    // Test invalid XML path
    let invalid_path = "invalid/path/to/xml";
    assert!(
        !path_exists(invalid_path),
        "XML file '{}' should not exist",
        invalid_path
    );
    assert_matches!(
        TestContextBuilder::simple()
            .with_config_file(invalid_path)
            .build(),
        Err(e) if e.is_entity_not_found(),
        "Connector should fail with invalid XML path"
    );

    // Test invalid participant profile (using Test.xml with invalid profile)
    assert_matches!(
        TestContextBuilder::simple()
            .with_config_name("InvalidParticipantProfile")
            .build(),
        Err(e) if e.is_entity_not_found(),
        "Connector should fail with invalid participant profile"
    );

    // Test valid instantiation
    assert_matches!(
        TestContextBuilder::simple().build(),
        Ok(_),
        "Connector should instantiate successfully"
    );
}

#[test]
fn test_multiple_connectors() {
    // Test multiple connectors with Test.xml
    const SIZE: usize = 3;
    let contexts = Vec::from_iter(
        std::iter::repeat_with(|| TestContextBuilder::simple().build())
            .take(SIZE)
            .map(|r| r.expect("TestContext should've been instantiated")),
    );
    assert_eq!(
        SIZE,
        contexts.len(),
        "Should create {} connectors, created {}",
        SIZE,
        contexts.len()
    );

    // Test for CON-163: Multiple Connector objects can be instantiated (using complex profile)
    const SIZE_2: usize = 2;
    let contexts_2 = Vec::from_iter(
        std::iter::repeat_with(|| TestContextBuilder::complex().build())
            .take(SIZE_2)
            .map(|r| Some(r.expect("TestContext should've been instantiated"))),
    );
    assert_eq!(
        SIZE_2,
        contexts_2.len(),
        "Should create {} connectors with complex profile, created {}",
        SIZE_2,
        contexts_2.len()
    );
}

#[test]
fn test_connector_advanced_features() {
    // Test loading multiple XML files
    fn combine_paths(mut final_string: String, file_path: &&str) -> String {
        assert!(
            path_exists(file_path),
            "XML file not found at {}",
            file_path
        );
        if !final_string.is_empty() {
            final_string.push(';');
        }
        final_string.push_str(file_path);
        final_string
    }

    let config_file = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources/Test.xml"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/shapes/Shapes.xml"),
    ]
    .iter()
    .fold(String::new(), combine_paths);

    // Use TestContext to properly set PARTITION_ID environment variable
    assert_matches!(
        TestContextBuilder::simple()
            .with_config_file(config_file)
            .build(),
        Ok(_),
        "Connector should instantiate successfully with multiple XML files"
    );

    // Test wait_for_data timeout
    let context = TestContextBuilder::simple_input_only()
        .build()
        .expect("Could not create test context for wait_for_data test");

    assert_matches!(
        context
            .connector
            .wait_for_data_with_timeout(std::time::Duration::from_millis(100)),
        Err(e) if e.is_timeout(),
        "Wait for data should timeout and return Err when no data is available"
    );
}

#[test]
fn test_connector_versions() {
    use regex::Regex;

    let versions = Connector::get_versions_string();

    assert!(
        Regex::new(r"^RTI Connector for Rust, version (\d+(?:\.\d+){2})")
            .unwrap()
            .is_match(&versions),
        "Version string should contain Connector for Rust version"
    );
    assert!(
        Regex::new(r".*NDDSCORE_BUILD_\d+(\.\d+){2,3}_\d{8}T\d{6}Z")
            .unwrap()
            .is_match(&versions),
        "Version string should contain nddscore build version"
    );
    assert!(
        Regex::new(r".*NDDSC_BUILD_\d+(\.\d+){2,3}_\d{8}T\d{6}Z")
            .unwrap()
            .is_match(&versions),
        "Version string should contain nddsc build version"
    );
    assert!(
        Regex::new(r".*RTICONNECTOR_BUILD_\d+(\.\d+){2,3}_\d{8}T\d{6}Z")
            .unwrap()
            .is_match(&versions),
        "Version string should contain rtiddsconnector build version"
    );
}

#[test]
fn test_selected_value_conversions() {
    // Test SelectedValue enum variants that might be returned by sample methods
    let number_val = SelectedValue::Number(42.0);
    let bool_val = SelectedValue::Boolean(true);
    let string_val = SelectedValue::String("test".to_string());

    assert_eq!(number_val, SelectedValue::Number(42.0));
    assert_eq!(bool_val, SelectedValue::Boolean(true));
    assert_eq!(string_val, SelectedValue::String("test".to_string()));

    // Test From implementations
    let from_number: SelectedValue = 42.0.into();
    let from_bool: SelectedValue = true.into();
    let from_string: SelectedValue = "test".to_string().into();
    let from_str: SelectedValue = "test".into();

    assert_eq!(from_number, SelectedValue::Number(42.0));
    assert_eq!(from_bool, SelectedValue::Boolean(true));
    assert_eq!(from_string, SelectedValue::String("test".to_string()));
    assert_eq!(from_str, SelectedValue::String("test".to_string()));

    // Test Clone and Debug
    let cloned = number_val.clone();
    assert_eq!(cloned, number_val);
    let _debug_str = format!("{:?}", number_val);
}
