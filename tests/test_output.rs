mod test_utils;

#[macro_use]
extern crate assert_matches;

use rtiddsconnector::ConnectorFallible;
use test_utils::TestContextBuilder;

//   it('Output object should not get instantiated for invalid DataWriter', function () {
//   it('Output object should get instantiated for valid ' +
#[test]
fn test_output_creation_and_display() -> ConnectorFallible {
    let context = TestContextBuilder::simple_output_only().build()?;
    let connector = &context.connector;

    // Test that trying to get/take an invalid Output fails
    assert_matches!(
        connector.get_output("InvalidPublisher::InvalidWriter"),
        Err(e) if e.is_entity_not_found(),
        "Expected error when getting invalid Output"
    );

    assert_matches!(
        connector.get_output("InvalidPublisher::InvalidWriter"),
        Err(e) if e.is_entity_not_found(),
        "Expected error when getting invalid Output"
    );

    // Test that we can get a valid Output, and we display its Debug representation
    let output = connector.get_output("TestPublisher::TestWriter")?;

    assert_eq!(
        r#"Output { name: "TestPublisher::TestWriter", parent: Connector { name: "TestDomainParticipantLibrary::SimpleParticipant" } }"#,
        format!("{:?}", output),
    );

    // Test that concurrent get succeeds
    assert_matches!(
        connector.get_output("TestPublisher::TestWriter"),
        Ok(_),
        "get_output should succeed on concurrent usage"
    );

    // Test that we can get a valid Output again after dropping
    drop(output);

    assert_matches!(
        connector.get_output("TestPublisher::TestWriter"),
        Ok(_),
        "Getting output after dropping taken output should succeed"
    );

    Ok(())
}

#[test]
fn test_output_basic_operations() -> ConnectorFallible {
    // Test the basic Output API without requiring actual subscriptions
    let context = TestContextBuilder::simple_output_only().build()?;
    let connector = &context.connector;

    let output = connector.get_output("TestPublisher::TestWriter")?;

    let mut instance = output.instance();

    assert_matches!(
        instance.set_number("long_field", 42.0),
        Ok(_),
        "Setting number field should succeed"
    );
    assert_matches!(
        instance.set_string("string_field", "test_value"),
        Ok(_),
        "Setting string field should succeed"
    );
    assert_matches!(
        instance.set_as_json(
            r#"{"long_field": 10, "double_field": 20.5, "string_field": "json_test"}"#
        ),
        Ok(_),
        "Setting from JSON should succeed"
    );

    // Note: We don't test write() here as it might require additional setup
    // The write functionality is tested in other integration tests
    Ok(())
}

#[test]
fn test_output_wait_operations() -> ConnectorFallible {
    let context = TestContextBuilder::simple_output_only().build()?;
    let connector = &context.connector;

    let output = connector.get_output("TestPublisher::TestWriter")?;

    assert_eq!(
        1,
        output.wait_for_subscriptions_with_timeout(std::time::Duration::from_secs(2))?,
    );

    // Test display of matched subscriptions
    assert_eq!(
        r#"[{"name":"TestReader"}]"#,
        output.display_matched_subscriptions()?,
    );

    assert_matches!(
        output.wait_with_timeout(std::time::Duration::from_secs(1)),
        Ok(_),
        "Wait with timeout shouldn't Err when no acknowledgments are received"
    );

    Ok(())
}

#[test]
fn test_output_instance_field_operations() -> ConnectorFallible {
    let context = TestContextBuilder::simple_output_only().build()?;
    let connector = &context.connector;

    let output = connector.get_output("TestPublisher::TestWriter")?;

    let mut instance = output.instance();
    // Test setting different field types
    instance.set_number("long_field", 100.0)?;
    instance.set_number("double_field", 200.75)?;
    instance.set_boolean("boolean_field", true)?;
    instance.set_string("string_field", "test_string")?;

    // Test setting with SelectedValue enum
    use rtiddsconnector::SelectedValue;
    instance.set_value("long_field", SelectedValue::Number(50.0))?;
    instance.set_value(
        "string_field",
        SelectedValue::String("another_value".to_string()),
    )?;

    // Test JSON operations
    let json_str = r#"{"long_field": 75, "double_field": 125.5, "boolean_field": false, "string_field": "json_string"}"#;
    instance.set_as_json(json_str)?;

    // Test instance display (should show JSON representation)
    let display = format!("{}", instance);
    assert!(
        display.contains("json_string"),
        "Display should contain the string_field we set: {}",
        display
    );
    assert!(
        display.contains("75"),
        "Display should contain the long_field value we set: {}",
        display
    );

    Ok(())
}

#[test]
fn test_output_wait_for_acknowledgments() -> ConnectorFallible {
    let context = TestContextBuilder::simple_output_only().build()?;
    let connector = &context.connector;

    let output = connector.get_output("TestPublisher::TestWriter")?;

    assert_matches!(
        output.wait_with_timeout(std::time::Duration::from_secs(1)),
        Ok(_),
        "Wait for acknowledgments should succeed"
    );

    Ok(())
}

#[test]
fn test_output_instance_display_and_operations() -> ConnectorFallible {
    let context = TestContextBuilder::simple_output_only().build()?;
    let connector = &context.connector;

    let output = connector.get_output("TestPublisher::TestWriter")?;

    let mut instance = output.instance();
    // Set various field types
    instance.set_number("long_field", 123.0)?;
    instance.set_number("double_field", 678.90)?;
    instance.set_boolean("boolean_field", true)?;
    instance.set_string("string_field", "test_display")?;

    let display = format!("{}", instance);

    // Verify the display contains our values
    assert!(
        display.contains(r#""string_field":"test_display""#),
        "Display should contain string_field: {}",
        display
    );
    assert!(
        display.contains("123"),
        "Display should contain long_field value: {}",
        display
    );
    assert!(
        display.contains("678"),
        "Display should contain double_field value: {}",
        display
    );
    assert!(
        display.contains("true"),
        "Display should contain boolean_field value: {}",
        display
    );

    // Test JSON round-trip
    let json_str = r#"{"long_field": 999, "double_field": 888.5, "boolean_field": false, "string_field": "updated"}"#;
    instance.set_as_json(json_str)?;

    let updated_display = format!("{}", instance);
    assert!(
        updated_display.contains("updated"),
        "Updated display should contain new string_field: {}",
        updated_display
    );
    assert!(
        updated_display.contains("999"),
        "Updated display should contain new long_field value: {}",
        updated_display
    );

    Ok(())
}
