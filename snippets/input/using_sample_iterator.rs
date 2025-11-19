use rtiddsconnector::{self, Input};

fn using_sample_iterator(input: &Input) -> rtiddsconnector::ConnectorFallible {
    for (i, sample) in input.into_iter().enumerate() {
        match sample.is_valid() {
            Ok(true) => {
                println!("Valid sample #{}: {}", i, sample)
            }

            Ok(false) => {
                println!("Sample #{} is invalid", i)
            }

            Err(e) => {
                println!("Error checking validity of sample #{}: {}", i, e)
            }
        }
    }

    Ok(())
}
