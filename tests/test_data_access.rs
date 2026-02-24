mod test_utils;

#[macro_use]
extern crate assert_matches;

use rtiddsconnector::SelectedValue;

use test_utils::TEST_TIMEOUT;

#[test]
fn test_input_invalidating_sample_on_read() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        output.write().expect("Failed to write data");
        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();
    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    let sample = iter.take(1).next().expect("Expected a sample");

    assert_matches!(
        sample.get_value("long_field"),
        Ok(SelectedValue::Number(0.0)),
        "Expected to be able to access sample value after read"
    );

    let mut input = input.clone();

    input.take().expect("Take operation failed");

    assert_matches!(
        sample.get_value("long_field"),
        Err(e) if e.is_stale_resource(),
        "Expected error when accessing sample value after take"
    );
}

#[test]
fn test_display_instance_and_sample() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let (input, output) = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        output.write().expect("Failed to write data");
        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        (input, output)
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();
    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    let instance = output.instance();
    let sample = iter.take(1).next().expect("Expected a sample");

    assert_eq!(
        format!("{}", instance),
        format!("{}", sample),
        "Instance and sample display outputs should match"
    );
}

#[test]
fn test_setget_value() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_value("long_field", 10_f64.into())
                .expect("Failed to set long_field");
            instance
                .set_value("double_field", 123.45.into())
                .expect("Failed to set double_field");
            instance
                .set_value("boolean_field", true.into())
                .expect("Failed to set boolean_field");
            instance
                .set_value("string_field", "Hello".into())
                .expect("Failed to set string_field");
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify data using type-variant accessors
        assert_eq!(
            SelectedValue::Number(10_f64),
            s.get_value("long_field")
                .expect("Failed 'get_value' operation on 'long_field'")
        );
        assert_eq!(
            SelectedValue::Number(123.45),
            s.get_value("double_field")
                .expect("Failed 'get_value' operation on 'double_field'")
        );
        assert_eq!(
            SelectedValue::Boolean(true),
            s.get_value("boolean_field")
                .expect("Failed 'get_value' operation on 'boolean_field'")
        );
        assert_eq!(
            SelectedValue::String("Hello".to_string()),
            s.get_value("string_field")
                .expect("Failed 'get_value' operation on 'string_field'")
        );
    }
}

// it('getNumber should return a number', () => {
// it('getNumber on a boolean field should return a number', () => {
// it('getNumber on an enum should return the set value', () => {
#[test]
fn test_setget_number() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_number("long_field", 10_f64)
                .expect("Failed to set long_field");
            instance
                .set_number("double_field", 123.45)
                .expect("Failed to set double_field");
            instance
                .set_number("enum_field", 1.0)
                .expect("Failed to set enum_field");
            instance
                .set_number("string_field", 123.0)
                .expect("Failed to set string_field"); // Implicit conversion
            instance
                .set_number("boolean_field", true as u64 as f64)
                .expect("Failed to set boolean_field"); // Implicit conversion
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify data using type-specific accessors
        assert_eq!(
            10_f64,
            s.get_number("long_field")
                .expect("Failed 'get_number' operation on 'long_field'")
        );
        assert_eq!(
            123.45,
            s.get_number("double_field")
                .expect("Failed 'get_number' operation on 'double_field'")
        );
        assert_eq!(
            1.0,
            s.get_number("enum_field")
                .expect("Failed 'get_number' operation on 'enum_field'")
        );
        assert_eq!(
            123.0,
            s.get_number("string_field") // Implicit conversion
                .expect("Failed 'get_number' operation on 'string_field'")
        );
        assert_eq!(
            1.0,
            s.get_number("boolean_field") // Implicit conversion
                .expect("Failed 'get_number' operation on 'boolean_field'")
        );
    }
}

// it('getBoolean should return a boolean', () => {
#[test]
fn test_setget_boolean() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_boolean("boolean_field", true)
                .expect("Failed to set boolean_field");
            instance
                .set_number("long_field", 1.0)
                .expect("Failed to set long_field"); // Implicit conversion to boolean
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify data using type-specific accessors
        assert!(
            s.get_boolean("boolean_field")
                .expect("Failed 'get_boolean' operation on 'boolean_field'")
        );
    }
}

// it('getString on a number field should return a string', () => {
// it('try to set a number with a string', () => {
// it('try to set a boolean with a string', () => {
#[test]
fn test_setget_string() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_string("string_field", "Hello")
                .expect("Failed to set string_field");
            instance
                .set_string("long_field", "123")
                .expect("Failed to set long_field"); // Implicit conversion
            instance
                .set_string("double_field", "123.45")
                .expect("Failed to set double_field"); // Implicit conversion
            instance
                .set_string("boolean_field", "true")
                .expect("Failed to set boolean_field"); // Implicit conversion
            instance
                .set_string("enum_field", "1")
                .expect("Failed to set enum_field"); // Implicit conversion
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify data using type-specific accessors
        assert_eq!(
            "Hello",
            s.get_string("string_field")
                .expect("Failed 'get_string' operation on 'string_field'")
        );
        assert_eq!(
            "123",
            s.get_string("long_field") // Implicit conversion
                .expect("Failed 'get_string' operation on 'long_field'")
        );
        assert_eq!(
            "123.45",
            &s.get_string("double_field") // Implicit conversion
                .expect("Failed 'get_string' operation on 'double_field'")[..6] // Truncate to avoid floating point precision issues
        );
        assert_eq!(
            "1",
            s.get_string("enum_field") // Implicit conversion
                .expect("Failed 'get_string' operation on 'enum_field'")
        );
    }
}

// it('getNumber requires a valid field name', () => {
// it('getString requires a valid field name', () => {
// it('getBoolean requires a valid field name', () => {
// it('getValue requires a valid field name', () => {
// it('attempt to access non-existent members', () => {
// it('try to set non-existent field names', () => {
#[test]
fn test_setget_invalid_field_or_value() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        // Attempt to set invalid field names or values
        {
            let mut instance = output.instance();
            assert_matches!(
                instance.set_number("non_existent_field", 10_f64),
                Err(e) if e.is_field_not_found(),
                "Unexpected OK when setting non-existent field using 'set_number'"
            );
            assert_matches!(
                instance.set_boolean("non_existent_field", true),
                Err(e) if e.is_field_not_found(),
                "Unexpected OK when setting non-existent field using 'set_boolean'"
            );
            assert_matches!(
                instance.set_string("non_existent_field", "Hello"),
                Err(e) if e.is_field_not_found(),
                "Unexpected OK when setting non-existent field using 'set_string'"
            );
            assert_matches!(
                instance.set_value("non_existent_field", 10_f64.into()),
                Err(e) if e.is_field_not_found(),
                "Unexpected OK when setting non-existent field using 'set_value'"
            );
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify that valid fields are still unset (default values)
        assert_eq!(
            0.0,
            s.get_number("long_field")
                .expect("Failed 'get_number' operation on 'long_field'")
        );
        assert_eq!(
            0.0,
            s.get_number("double_field")
                .expect("Failed 'get_number' operation on 'double_field'")
        );
        assert!(
            !s.get_boolean("boolean_field")
                .expect("Failed 'get_boolean' operation on 'boolean_field'")
        );
        assert_eq!(
            "",
            s.get_string("string_field")
                .expect("Failed 'get_string' operation on 'string_field'")
        );
        assert_matches!(
            s.get_number("non_existent_field"),
            Err(e) if e.is_field_not_found(),
            "Expected error for non-existent field"
        );
        assert_matches!(
            s.get_boolean("non_existent_field"),
            Err(e) if e.is_field_not_found(),
            "Expected error for non-existent field"
        );
        assert_matches!(
            s.get_string("non_existent_field"),
            Err(e) if e.is_field_not_found(),
            "Expected error for non-existent field"
        );
        assert_matches!(
            s.get_value("non_existent_field"),
            Err(e) if e.is_field_not_found(),
            "Expected error for non-existent field"
        );
    }
}

#[test]
fn test_get_info_fields() {
    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();
    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    let sample = iter.take(1).next().expect("Expected a sample");

    // Test that unknown fields return an error
    assert_matches!(
        sample.get_info("unknown_field"),
        Err(e) if e.is_field_not_found(),
        "Expected error for unknown info field"
    );
    assert_matches!(
        sample.get_info_json("unknown_field"),
        Err(e) if e.is_field_not_found(),
        "Expected error for unknown info field with get_info_json"
    );

    // Test valid_data field
    assert_matches!(
        sample.get_info("valid_data"),
        Ok(SelectedValue::Boolean(true)),
        "Expected 'valid_data' info field to be true"
    );
    assert_matches!(
        sample.get_info_json("valid_data"),
        Err(_),
        "A boolean can't be turned into a JSON value"
    );

    // Test source_timestamp field (should return a value, exact format may vary)
    let selected = sample
        .get_info("source_timestamp")
        .expect("Expected 'source_timestamp' info field to be present");
    let json = sample
        .get_info_json("source_timestamp")
        .expect("Expected 'source_timestamp' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'source_timestamp' to be a string"
    );

    // Test reception_timestamp field
    let selected = sample
        .get_info("reception_timestamp")
        .expect("Expected 'reception_timestamp' info field to be present");
    let json = sample
        .get_info_json("reception_timestamp")
        .expect("Expected 'reception_timestamp' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'reception_timestamp' to be a string"
    );

    // Test sample_identity field
    let selected = sample
        .get_info("sample_identity")
        .expect("Expected 'sample_identity' info field to be present");
    let json = sample
        .get_info_json("sample_identity")
        .expect("Expected 'sample_identity' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'sample_identity' to be a string"
    );

    // Test related_sample_identity field
    let selected = sample
        .get_info("related_sample_identity")
        .expect("Expected 'related_sample_identity' info field to be present");
    let json = sample
        .get_info_json("related_sample_identity")
        .expect("Expected 'related_sample_identity' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'related_sample_identity' to be a string"
    );

    // Test sample_state field
    let selected = sample
        .get_info("sample_state")
        .expect("Expected 'sample_state' info field to be present");
    let json = sample
        .get_info_json("sample_state")
        .expect("Expected 'sample_state' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'sample_state' to be a string"
    );

    // Test view_state field
    let selected = sample
        .get_info("view_state")
        .expect("Expected 'view_state' info field to be present");
    let json = sample
        .get_info_json("view_state")
        .expect("Expected 'view_state' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'view_state' to be a string"
    );

    // Test instance_state field
    let selected = sample
        .get_info("instance_state")
        .expect("Expected 'instance_state' info field to be present");
    let json = sample
        .get_info_json("instance_state")
        .expect("Expected 'instance_state' to be available via get_info_json");
    assert_matches!(
        selected,
        SelectedValue::String(value) if value == json,
        "Expected 'instance_state' to be a string"
    );
}

// it('getString requires a valid index', () => {
// it('getBoolean requires a valid index', () => {
// it('getValue requires a valid index', () => {
#[test]
#[ignore = "index access is internal to the Input and not yet exposed"]
fn test_setget_by_index() {}

// it('access a value nested within a struct', () => {
#[test]
fn test_output_access_a_value_nested_within_a_struct() {
    let mut context = test_utils::TestContextBuilder::complex()
        .build()
        .expect("Failed to create test context");

    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_number("simple.long_field", 10_f64)
                .expect("Failed to set simple.long_field");
            instance
                .set_number("simple.double_field", 123.45)
                .expect("Failed to set simple.double_field");
            instance
                .set_string("simple.string_field", "Hello")
                .expect("Failed to set simple.string_field");
            instance
                .set_boolean("simple.boolean_field", true)
                .expect("Failed to set simple.boolean_field");
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify data using type-variant accessors
        assert_eq!(
            SelectedValue::Number(10_f64),
            s.get_value("simple.long_field")
                .expect("Failed 'get_value' operation on 'simple.long_field'")
        );
        assert_eq!(
            SelectedValue::Number(123.45),
            s.get_value("simple.double_field")
                .expect("Failed 'get_value' operation on 'simple.double_field'")
        );
        assert_eq!(
            SelectedValue::String("Hello".to_string()),
            s.get_value("simple.string_field")
                .expect("Failed 'get_value' operation on 'simple.string_field'")
        );
        assert_eq!(
            SelectedValue::Boolean(true),
            s.get_value("simple.boolean_field")
                .expect("Failed 'get_value' operation on 'simple.boolean_field'")
        );

        // Verify data using type-specific accessors
        assert_eq!(
            10_f64,
            s.get_number("simple.long_field")
                .expect("Failed 'get_number' operation on 'simple.long_field'")
        );
        assert_eq!(
            123.45,
            s.get_number("simple.double_field")
                .expect("Failed 'get_number' operation on 'simple.double_field'")
        );
        assert_eq!(
            "Hello",
            s.get_string("simple.string_field")
                .expect("Failed 'get_string' operation on 'simple.string_field'")
        );
        assert!(
            s.get_boolean("simple.boolean_field")
                .expect("Failed 'get_boolean' operation on 'simple.boolean_field'")
        );
    }
}

fn combinatorial(range: std::ops::Range<usize>) -> impl Iterator<Item = (usize, usize)> {
    (range)
        .clone()
        .flat_map(move |r| range.clone().map(move |c| (r, c)))
}

// it('access values and sizes of sequences and arrays', () => {
// it('access values past the end of a sequence', () => {
// it('attempt to access members with bad sequence syntax', () => {
// it('attempt to access the negative member of a sequence', () => {
#[test]
fn test_setget_array_and_sequences() {
    const MATRIX_SIZE: usize = 3;
    const SEQUENCE_SIZE: usize = 3;

    let mut context = test_utils::TestContextBuilder::complex()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            for (row, col) in combinatorial(0..MATRIX_SIZE) {
                let value = (row + col) as f64;
                // Flat indexing
                instance
                    .set_number(&format!("long_matrix[{row},{col}]"), value)
                    .expect("Failed to set long_matrix");
                // Nested indexing
                // TODO: Review nested indexing
                // instance.set_number(&format!("long_matrix[{row}][{col}]"), value)?;
            }

            // Out of bounds access should be an error
            assert_matches!(
                instance.set_number(
                    &format!("long_matrix[{MATRIX_SIZE},{MATRIX_SIZE}]"),
                    0.0
                ),
                Err(_),
                "Expected error for out-of-bounds access"
            );

            for index in 0..SEQUENCE_SIZE {
                instance
                    .set_number(
                        &format!("double_sequence[{index}]"),
                        123.45 * (index + 1) as f64,
                    )
                    .expect("Failed to set double_sequence");
            }

            // Out of bounds access should be an error
            assert_matches!(
                instance.set_number(&format!("double_sequence[{SEQUENCE_SIZE}]"), 0.0),
                Err(_),
                "Expected error for out-of-bounds access"
            );
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        for (row, col) in combinatorial(0..MATRIX_SIZE) {
            let value = (row + col) as f64;
            let flat_field = format!("long_matrix[{row},{col}]");
            // TODO: Review nested indexing
            // let nested_field = format!("long_matrix[{row}][{col}]");

            // Verify data using type-variant accessors
            assert_eq!(
                value,
                s.get_number(&flat_field)
                    .expect("Failed 'get_number' operation on 'long_matrix[row,col]'")
            );
            // assert_eq!(
            //     value,
            //     s.get_number(&nested_field)
            //         .expect("Failed 'get_number' operation on 'long_matrix[row][col]'")
            // );

            // Verify data using type-specific accessors
            assert_eq!(
                SelectedValue::Number(value),
                s.get_value(&flat_field)
                    .expect("Failed 'get_value' operation on 'long_matrix[row,col]'")
            );
            // assert_eq!(
            //     SelectedValue::Number(value),
            //     s.get_value(&nested_field)
            //         .expect("Failed 'get_value' operation on 'long_matrix[row][col]'")
            // );
        }

        // Out of bounds access should be an error
        assert_matches!(
            s.get_number("long_matrix[-1][-1]"),
            Err(_),
            "Expected error for out-of-bounds access (negative index)"
        );
        assert_matches!(
            s.get_number("long_matrix[bad,index]"),
            Err(_),
            "Expected error for out-of-bounds access (non-numeric index)"
        );
        assert_matches!(
            s.get_number(&format!("long_matrix[{MATRIX_SIZE},{MATRIX_SIZE}]")),
            Err(_),
            "Expected error for out-of-bounds access"
        );

        for index in 0..SEQUENCE_SIZE {
            let value = 123.45 * (index + 1) as f64;
            let field = format!("double_sequence[{index}]");

            // Verify data using type-variant accessors
            assert_eq!(
                value,
                s.get_number(&field)
                    .expect("Failed 'get_number' operation on 'double_sequence[index]'")
            );

            // Verify data using type-specific accessors
            assert_eq!(
                SelectedValue::Number(value),
                s.get_value(&field)
                    .expect("Failed 'get_value' operation on 'double_sequence[index]'")
            );
        }

        // Out of bounds access should be an error
        assert_matches!(
            s.get_number("double_sequence[-1]"),
            Err(_),
            "Expected error for out-of-bounds access (negative index)"
        );
        assert_matches!(
            s.get_number("double_sequence[bad_index]"),
            Err(_),
            "Expected error for out-of-bounds access (negative index)"
        );
        assert_matches!(
            s.get_number(&format!("double_sequence[{SEQUENCE_SIZE}]")),
            Err(_),
            "Expected error for out-of-bounds access"
        );
    }
}

// it('obtain the selected member of a union with # syntax', () => {
// it('getNumber on unions', () => {
#[test]
fn test_setget_unions() {
    let mut context = test_utils::TestContextBuilder::complex()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_string("union.string", "Hello")
                .expect("Failed to set union.string");
        }
        output
            .write()
            .expect("Failed to write data for 'union# == string'");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        {
            let mut instance = output.instance();
            instance
                .set_number("union.number", 123.45)
                .expect("Failed to set union.number");
        }
        output
            .write()
            .expect("Failed to write data for 'union# == number'");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        {
            let mut instance = output.instance();
            instance
                .set_boolean("union.boolean", true)
                .expect("Failed to set union.boolean");
        }
        output
            .write()
            .expect("Failed to write data for 'union# == boolean'");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        3,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only three samples"
    );

    for s in iter.take(3) {
        let discriminator = s
            .get_string("union#")
            .expect("Failed to get 'union#' value");

        match discriminator.as_str() {
            "string" => {
                assert_eq!(
                    "Hello",
                    s.get_string("union.string")
                        .expect("Failed 'get_string' operation on 'union.string'")
                );
                assert_matches!(
                    s.get_value("union.number"),
                    Err(_),
                    "Expected error for inactive union member 'union.number'"
                );
                assert_matches!(
                    s.get_value("union.boolean"),
                    Err(_),
                    "Expected error for inactive union member 'union.boolean'"
                );
            }
            "number" => {
                assert_eq!(
                    123.45,
                    s.get_number("union.number")
                        .expect("Failed 'get_number' operation on 'union.number'")
                );
                assert_matches!(
                    s.get_value("union.string"),
                    Err(_),
                    "Expected error for inactive union member 'union.string'"
                );
                assert_matches!(
                    s.get_value("union.boolean"),
                    Err(_),
                    "Expected error for inactive union member 'union.boolean'"
                );
            }
            "boolean" => {
                assert!(
                    s.get_boolean("union.boolean")
                        .expect("Failed 'get_boolean' operation on 'union.boolean'")
                );
                assert_matches!(
                    s.get_value("union.string"),
                    Err(_),
                    "Expected error for inactive union member 'union.string'"
                );
                assert_matches!(
                    s.get_value("union.number"),
                    Err(_),
                    "Expected error for inactive union member 'union.number'"
                );
            }
            _ => panic!("Unexpected union member"),
        }
    }
}

// it('obtain an unset optional member', () => {
// it('obtain an unset optional member as a string', () => {
// it('obtain an unset optional complex member', () => {
#[test]
fn test_unset_optional_members() {
    let mut context = test_utils::TestContextBuilder::complex()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        {
            let mut instance = output.instance();
            instance
                .set_number("optional.long_field", 10_f64)
                .expect("Failed to set optional.long_field");
            instance
                .set_number("optional.double_field", 123.45)
                .expect("Failed to set optional.double_field");
            instance
                .set_boolean("optional.boolean_field", true)
                .expect("Failed to set optional.boolean_field");
            // Explicitly clear optional fields
            instance
                .clear("optional.string_field")
                .expect("Failed to clear optional.string_field");
            instance
                .clear("optional.enum_field")
                .expect("Failed to clear optional.enum_field");
        }
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    for s in iter.take(1) {
        // Verify that optional fields are unset (default values)
        assert_eq!(
            10_f64,
            s.get_number("optional.long_field")
                .expect("Failed 'get_number' operation on 'optional.long_field'")
        );
        assert_eq!(
            123.45,
            s.get_number("optional.double_field")
                .expect("Failed 'get_number' operation on 'optional.double_field'")
        );
        assert!(
            s.get_boolean("optional.boolean_field")
                .expect("Failed 'get_boolean' operation on 'optional.boolean_field'")
        );
        assert_matches!(
            s.get_string("optional.string_field"),
            Err(_),
            "Expected error for unset optional string field"
        );
        assert_matches!(
            s.get_value("optional.enum_field"),
            Err(_),
            "Expected error for unset optional enum field"
        );
    }
}

#[test]
fn test_sample_get_json() {
    let mut context = test_utils::TestContextBuilder::complex()
        .build()
        .expect("Failed to create test context");
    let input = {
        let entities = context
            .test_entities()
            .expect("Error in test entities creation")
            .ensure_discovery();
        let mut output = entities
            .output
            .expect("This test expects an available output");
        let mut input = entities
            .input
            .expect("This test expects an available input");

        let mut instance = output.instance();
        instance
            .set_number("simple.long_field", 10_f64)
            .expect("Failed to set field");
        instance
            .set_number("simple.double_field", 123.45)
            .expect("Failed to set field");
        instance
            .set_number("simple.enum_field", 1.0)
            .expect("Failed to set field");
        instance
            .set_string("simple.string_field", "Hello")
            .expect("Failed to set field");
        instance
            .set_boolean("simple.boolean_field", true)
            .expect("Failed to set field");
        output.write().expect("Failed to write data");

        input
            .wait_with_timeout(TEST_TIMEOUT)
            .expect("Failed to wait for data");
        input.read().expect("Failed to read data");

        input
    };

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();

    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    let sample = iter.take(1).next().expect("Expected a sample");

    let json = sample
        .get_value_json("simple")
        .expect("Failed 'get_value_json' operation on the sample");

    // It's not trivial to build the expected JSON string manually, so we use
    // serde_json to build it for us.
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SimpleStruct {
        long_field: i64,
        double_field: f64,
        enum_field: i32,
        string_field: String,
        boolean_field: bool,
    }

    let expected_json = {
        let mut value = serde_json::to_value(SimpleStruct {
            long_field: 10,
            double_field: 123.45,
            enum_field: 1,
            string_field: "Hello".to_string(),
            boolean_field: true,
        })
        .expect("Failed to serialize expected value to JSON Value");
        value.sort_all_objects();
        value.to_string()
    };

    let json = {
        let mut value = serde_json::from_str::<serde_json::Value>(&json)
            .expect("Failed to parse JSON representation from get_value_json");
        value.sort_all_objects();
        value.to_string()
    };

    assert_eq!(
        expected_json, json,
        "JSON representation does not match expected"
    );
}

#[test]
fn test_write_with_params() {
    let mut context = test_utils::TestContextBuilder::complex()
        .build()
        .expect("Failed to create test context");
    let entities = context
        .test_entities()
        .expect("Error in test entities creation")
        .ensure_discovery();

    let mut output = entities
        .output
        .expect("This test expects an available output");
    let mut input = entities
        .input
        .expect("This test expects an available input");

    let mut instance = output.instance();
    instance
        .set_string("simple.string_field", "Hello")
        .expect("Failed to set field");

    output.write().expect("Failed to write with parameters");

    input
        .wait_with_timeout(TEST_TIMEOUT)
        .expect("Failed to wait for data");

    input.take().expect("Failed to read data");

    let iter = input.into_iter().valid_only();
    let (_, upper_hint) = iter.size_hint();
    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample"
    );

    let sample = iter.take(1).next().expect("Expected a sample");

    assert_eq!(
        "Hello",
        sample
            .get_string("simple.string_field")
            .expect("Failed to get 'simple.string_field' value")
    );

    output
        .write_with_params(&rtiddsconnector::WriteParams::dispose())
        .expect("Failed to write with dispose parameters");

    input
        .wait_with_timeout(TEST_TIMEOUT)
        .expect("Failed to wait for data after dispose");

    input.take().expect("Failed to read data after dispose");

    let iter = input.into_iter();
    assert_eq!(1, iter.len(), "Expected one (invalid) sample after dispose");

    let iter = iter.valid_only();
    let (_, upper_hint) = iter.size_hint();
    assert_eq!(
        1,
        upper_hint.expect("Our implementation always yields a upper hint"),
        "Expected only one sample after dispose"
    );

    assert_matches!(
        iter.take(1).next(),
        None,
        "Expected instance to be disposed"
    );
}

#[test]
fn test_typed_serialize_and_deserialize() {
    use test_utils::types::SimpleStruct;

    let mut context = test_utils::TestContextBuilder::simple()
        .build()
        .expect("Failed to create test context");
    let entities = context
        .test_entities()
        .expect("Failed to get test entities")
        .ensure_discovery();

    let mut output = entities
        .output
        .expect("Output should be available in test context");
    let mut input = entities
        .input
        .expect("Input should be available in test context");

    // Create a simple struct to serialize
    let original_data = SimpleStruct {
        long_field: 42,
        double_field: 123.45,
        boolean_field: true,
        string_field: "Hello, DDS!".to_string(),
        enum_field: test_utils::types::TestEnum::Green,
    };

    // Serialize and write the data
    output
        .instance()
        .serialize(&original_data)
        .expect("Failed to serialize data");

    output.write().expect("Failed to write data");

    // Wait for data to be available
    input
        .wait_with_timeout(TEST_TIMEOUT)
        .expect("Failed to wait for data");

    // Take the data and read it
    input.take().expect("Failed to take samples");

    // Get the first sample
    let sample = input
        .into_iter()
        .valid_only()
        .next()
        .expect("Expected at least one valid sample");

    // Deserialize the data
    let deserialized_data: SimpleStruct =
        sample.deserialize().expect("Failed to deserialize data");

    // Verify the deserialized data matches the original
    assert_eq!(
        original_data, deserialized_data,
        "Deserialized data should match original"
    );
}
