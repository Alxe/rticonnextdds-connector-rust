use rtiddsconnector::{self, Input, Output, SelectedValue};

fn using_selected_value(
    input: &mut Input,
    output: &mut Output,
) -> rtiddsconnector::ConnectorFallible {
    output.instance().set_value("field_name", true.into())?;
    output.write()?;

    // Ensure that the input has received the data by waiting
    input.take()?;

    for s in input.into_iter().take(1) {
        assert!(matches!(
            s.get_value("field_name")?,
            SelectedValue::Boolean(true)
        ));
    }

    Ok(())
}
