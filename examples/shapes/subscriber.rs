// Subscriber functionality

use super::{
    INPUT_NAME, SUB_PARTICIPANT_NAME as PARTICIPANT_NAME, TypedMode, config_path,
};

use rtiddsconnector::Connector;

macro_rules! tlog {
        ($fmt:expr) => {
            println!("[Sub] {}", $fmt)
        };
        ($fmt:expr, $($arg:tt)*) => {
            println!("[Sub] {}", format!($fmt, $($arg)*))
        };
    }

pub fn main(
    typed_mode: TypedMode,
    samples: usize,
    wait_ms: u64,
    wait_for_publications_ms: u64,
) -> super::Result<()> {
    let config_path = config_path()?;

    tlog!(
        "Loading subscriber configuration: file={}, participant={}, input={}",
        config_path.display(),
        PARTICIPANT_NAME,
        INPUT_NAME
    );

    let connector = Connector::new(PARTICIPANT_NAME, &config_path.to_string_lossy())?;
    let wait_timeout = super::optional_duration_from_ms(wait_ms);
    let discovery_duration = super::optional_duration_from_ms(wait_for_publications_ms);

    tlog!("Started subscriber...");

    let mut input = connector
        .get_input(INPUT_NAME)
        .map_err(|e| format!("Failed to take input: {}", e))?;

    loop {
        let wait_result = match discovery_duration {
            Some(timeout) => input.wait_for_publications_with_timeout(timeout),
            None => input.wait_for_publications(),
        };

        match wait_result {
            Ok(count) => {
                tlog!(
                    "Discovered {} publications, proceeding to subscribe...",
                    count
                );
                break;
            }
            Err(e) if e.is_timeout() => {
                tlog!("No publications discovered yet, retrying...");
            }
            Err(e) => {
                return Err(format!("Wait for publications failed: {}", e).into());
            }
        }
    }

    let mut samples_read = 0;

    while samples_read < samples {
        tlog!("Waiting for data...");

        let wait_result = match wait_timeout {
            Some(timeout_duration) => input.wait_with_timeout(timeout_duration),
            None => input.wait(),
        };

        match wait_result {
            Ok(_) => {
                tlog!("Data available, reading samples...");
            }
            Err(e) if e.is_timeout() => {
                tlog!("Wait timed out, no data available yet.");
                continue; // Retry waiting
            }
            Err(e) => {
                return Err(format!("Wait failed: {}", e).into());
            }
        }

        input
            .take()
            .map_err(|e| format!("Failed to take samples: {}", e))?;

        for s in input.into_iter().valid_only() {
            samples_read += 1;

            let sample_string: String = match typed_mode {
                TypedMode::Enabled => {
                    let shape: super::ShapeType = s.deserialize()?;

                    format!(
                        "Shape {{ x: {}, y: {}, shapesize: {}, color: '{}' }}",
                        shape.x, shape.y, shape.shapesize, shape.color
                    )
                }
                TypedMode::Disabled => {
                    let x = s.get_number("x")?;
                    let y = s.get_number("y")?;
                    let shapesize = s.get_number("shapesize")?;
                    let color = s.get_string("color")?;

                    format!(
                        "Shape {{ x: {}, y: {}, shapesize: {}, color: '{}' }}",
                        x, y, shapesize, color
                    )
                }
            };

            tlog!("Sample #{}: {}", samples_read, sample_string);

            if samples_read >= samples {
                break;
            }
        }
    }

    tlog!("Completed {} samples, exiting...", samples);
    tlog!("Subscriber completed successfully!");
    Ok(())
}
