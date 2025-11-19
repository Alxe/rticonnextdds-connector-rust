use rtiddsconnector::{self, Output};

fn using_output(
    output: &mut Output,
    samples_to_write: usize,
) -> rtiddsconnector::ConnectorFallible {
    let mut samples_written: usize = 0;

    while samples_written < samples_to_write {
        let mut instance = output.instance();
        instance.set_string("color", "red")?;
        instance.set_number("x", 10.0)?;
        instance.set_number("y", 20.0)?;

        // Write a sample
        output.write()?;

        match output.wait_with_timeout(std::time::Duration::from_secs(5)) {
            Ok(()) => {
                println!(
                    "Written sample #{}, acknowledged by {}",
                    samples_written,
                    output.display_matched_subscriptions()?
                );
                samples_written += 1;
            }

            Err(e) if e.is_timeout() => {
                println!(
                    "Written sample #{} but timeout waiting for acknowledgments from {}",
                    samples_written,
                    output.display_matched_subscriptions()?
                );
            }

            Err(e) => {
                println!("Timeout waiting for acknowledgments: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
