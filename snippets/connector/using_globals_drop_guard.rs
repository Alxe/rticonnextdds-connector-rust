use rtiddsconnector::{self, Connector};

fn use_globals_drop_guard() {
    let _globals = rtiddsconnector::GlobalsDropGuard;

    // This leads to the initialization of Connext globals, even if failed
    assert!(
        Connector::new("ThisParticipant::DoesNotExist", "nonexistent.xml").is_err(),
        "Expected error when creating Connector with invalid parameters"
    );

    // When `_globals` goes out of scope, Connext globals will be finalized
}
