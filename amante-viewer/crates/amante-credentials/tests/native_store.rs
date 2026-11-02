//! Opt-in native-store test. Never run against the user's normal session.
//! Linux: scripts/test-native-credentials.sh creates a private bus/keyring.
use amante_credentials::{Entry, Error};

#[test]
#[ignore = "requires an explicitly isolated OS credential store"]
fn native_store_round_trip_and_service_isolation() {
    assert_eq!(std::env::var("AMANTE_ISOLATED_KEYRING_TEST").as_deref(), Ok("1"));
    let service = format!("Amante isolated test {}", std::process::id());
    let entry = Entry::new(&service, "synthetic-resident").unwrap();
    let other = Entry::new(&format!("{service} other"), "synthetic-resident").unwrap();
    assert!(matches!(entry.get_password(), Err(Error::NoEntry)));
    entry.set_password("synthetic-test-only-ä🔒").unwrap();
    assert_eq!(entry.get_password().unwrap(), "synthetic-test-only-ä🔒");
    assert!(matches!(other.get_password(), Err(Error::NoEntry)));
    // A fresh handle must find the saved entry under the same identifiers.
    assert_eq!(
        Entry::new(&service, "synthetic-resident").unwrap().get_password().unwrap(),
        "synthetic-test-only-ä🔒"
    );
    entry.set_password("updated-test-only").unwrap();
    assert_eq!(entry.get_password().unwrap(), "updated-test-only");
    entry.delete_credential().unwrap();
    assert!(matches!(entry.get_password(), Err(Error::NoEntry)));
}
