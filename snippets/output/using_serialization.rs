use rtiddsconnector::{self, Instance};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct ShapeType {
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

fn using_serialization(instance: &mut Instance) -> rtiddsconnector::ConnectorFallible {
    let shape = ShapeType {
        color: "BLUE".to_string(),
        x: 100,
        y: 150,
        shapesize: 30,
    };

    println!("Serializing shape: {:?}", shape);

    instance.serialize(&shape)
}
