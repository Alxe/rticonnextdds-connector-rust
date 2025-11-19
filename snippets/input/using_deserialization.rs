use rtiddsconnector::{self, Sample};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ShapeType {
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

fn using_deserialization(
    sample: &mut Sample,
) -> rtiddsconnector::ConnectorResult<ShapeType> {
    println!("Deserializing sample: {}", sample);

    let shape: ShapeType = sample.deserialize()?;
    println!("Deserialized shape: {:?}", shape);

    Ok(shape)
}
