use rtiddsconnector::{self, Connector};

fn using_connector(
    resources_path: &std::path::Path,
) -> rtiddsconnector::ConnectorFallible {
    let config_file = resources_path.join("App.xml");
    let config_file = config_file.to_str().unwrap();

    let connector = Connector::new("App::Participant", config_file)?;
    println!("Connector created: {:?}", connector);

    let input = connector.get_input("Sub::Reader")?;
    println!("Input acquired: {:?}", input);

    let output = connector.get_output("Pub::Writer")?;
    println!("Output acquired: {:?}", output);

    Ok(())
}
