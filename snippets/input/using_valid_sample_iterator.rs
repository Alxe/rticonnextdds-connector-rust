use rtiddsconnector::{self, Input};

fn using_sample_iterator(input: &Input) -> rtiddsconnector::ConnectorFallible {
    for (i, sample) in input.into_iter().valid_only().enumerate() {
        println!("Valid sample #{}: {}", i, sample)
    }

    Ok(())
}
