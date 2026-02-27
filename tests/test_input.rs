mod test_utils;

#[macro_use]
extern crate assert_matches;

use test_utils::TestContextBuilder;

//   it('Input object should not get instantiated for invalid DataReader', function () {
//   it('Input object should get instantiated for valid ' +
#[test]
fn test_input_creation_and_display() {
    let context = TestContextBuilder::simple_input_only()
        .build()
        .expect("Failed to create test context");
    let connector = &context.connector;

    // Test that trying to get/take an invalid Input fails
    assert_matches!(
        connector.get_input("InvalidSubscriber::InvalidReader"),
        Err(e) if e.is_entity_not_found(),
        "Expected error when getting invalid Input"
    );

    assert_matches!(
        connector.take_input("InvalidSubscriber::InvalidReader"),
        Err(e) if e.is_entity_not_found(),
        "Expected error when getting invalid Input"
    );

    // Test that we can get a valid Input, and we display its Debug representation
    let input = connector
        .take_input("TestSubscriber::TestReader")
        .expect("Failed to get valid Input");

    assert_eq!(
        r#"Input { name: "TestSubscriber::TestReader", parent: Connector { name: "TestDomainParticipantLibrary::SimpleParticipant" } }"#,
        format!("{:?}", input),
    );

    // Test that concurrent get fails
    assert_matches!(
        connector.get_input("TestSubscriber::TestReader"),
        Err(_),
        "get_input should fail on concurrent usage"
    );

    // Test that we can get a valid Input again after dropping
    drop(input);

    assert_matches!(
        connector.get_input("TestSubscriber::TestReader"),
        Ok(_),
        "Getting input after dropping taken input should succeed"
    );
}

#[test]
fn test_input_basic_operations_no_data() {
    // Test the basic Input API without requiring actual data
    let context = TestContextBuilder::simple_input_only()
        .build()
        .expect("Failed to create test context");
    let connector = &context.connector;

    let mut input = connector
        .get_input("TestSubscriber::TestReader")
        .expect("Failed to get valid Input");

    assert_matches!(
        input.read(),
        Ok(_),
        "Read operation should succeed (may return 0 samples)"
    );
    assert_matches!(
        input.take(),
        Ok(_),
        "Take operation should succeed (may return 0 samples)"
    );
    assert_matches!(
        input.return_loan(),
        Ok(_),
        "Return loan operation should always succeed"
    );

    assert_eq!(
        1,
        input
            .wait_for_publications_with_timeout(std::time::Duration::from_secs(2))
            .expect("Wait for publications failed"),
    );

    // Test display of matched publications
    assert_eq!(
        r#"[{"name":"TestWriter"}]"#,
        input
            .display_matched_publications()
            .expect("Failed to get matched publications")
    );

    assert_matches!(
        input.wait_with_timeout(std::time::Duration::from_secs(1)),
        Err(_),
        "Wait with timeout should return Err when no publications are matched"
    );
}

#[test]
fn test_sample_iterator_basic() {
    let context = TestContextBuilder::simple_input_only()
        .build()
        .expect("Failed to create test context");
    let connector = &context.connector;

    let input = connector
        .get_input("TestSubscriber::TestReader")
        .expect("Failed to get valid Input");

    if input.into_iter().next().is_some() {
        panic!("Iterator should be empty, and this shouldn't happen");
    }

    if input.into_iter().valid_only().next().is_some() {
        panic!("Iterator should be empty, and this shouldn't happen");
    }

    // Test iterator creation and basic operations
    let mut iter = input.into_iter();

    // Test length method
    let length = iter.len();
    assert_eq!(0, length, "Empty iterator should have length 0");

    // Test size hint
    let (lower, upper) = iter.size_hint();
    let upper = upper.expect("Should have an upper bound");
    assert_eq!(length, lower, "len() should match size hint");
    assert_eq!(length, upper, "len() should match size hint");
    assert_eq!(
        lower, upper,
        "Lower and upper should be equal for ExactSizeIterator"
    );

    // Test next method
    assert!(
        iter.next().is_none(),
        "Iterator should be empty, next() should return None"
    );
}
