use rtiddsconnector::{self, Input};

fn using_input(
    input: &Input,
    samples_to_process: usize,
) -> rtiddsconnector::ConnectorFallible {
    let mut processed_samples: usize = 0;

    while processed_samples < samples_to_process {
        match input.wait_with_timeout(std::time::Duration::from_secs(10)) {
            Ok(_) => {
                println!("Data available!");
            }

            Err(e) if e.is_timeout() => {
                println!("No data received in the last 10 seconds, waiting again...");
                continue;
            }

            Err(e) => {
                println!("Error or timeout while waiting for data: {}", e);
                return Err(e);
            }
        };

        // Iterate over the samples
        for sample in input.into_iter().valid_only() {
            println!("Received sample #{}: {}", processed_samples, sample);

            processed_samples += 1;
        }
    }

    Ok(())
}
