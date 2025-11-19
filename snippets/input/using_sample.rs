use rtiddsconnector::{self, Sample};

/// Assuming we get a ShapeType sample and prints its fields.
fn using_sample(s: &Sample<'_>) -> rtiddsconnector::ConnectorFallible {
    println!(
        "Shape sample - x: {}, y: {}, color: {}",
        s.get_number("x")?,
        s.get_number("y")?,
        s.get_string("color")?
    );

    println!("Alternatively, print the whole sample: {}", s);

    Ok(())
}
