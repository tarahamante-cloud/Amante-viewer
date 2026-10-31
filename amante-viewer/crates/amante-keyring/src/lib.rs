//! OS-backed credentials with stable service/user names across backend changes.
//!
//! Windows Credential Manager and macOS Keychain are unchanged. Unix uses the
//! maintained DBus Secret Service backend, searching the same `service` and
//! `username` attributes in the same default collection as the former ZBus
//! backend. There is no file/plaintext fallback and no secret migration/copy.

pub use keyring_core::{Entry, Error, Result};
use std::sync::LazyLock;

static INITIALIZED: LazyLock<Result<()>> = LazyLock::new(initialize);

/// Construct an entry only after successfully opening the OS credential store.
pub fn entry(service: &str, username: &str) -> Result<Entry> {
    if INITIALIZED.is_err() {
        return Err(Error::NoDefaultStore);
    }
    Entry::new(service, username)
}

fn initialize() -> Result<()> {
    #[cfg(target_os = "windows")]
    let store = windows_native_keyring_store::Store::new()?;
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
    let store = dbus_secret_service_keyring_store::Store::new()?;
    #[cfg(all(any(unix, windows), not(any(target_os = "ios", target_os = "android"))))]
    {
        keyring_core::set_default_store(store);
        Ok(())
    }
    #[cfg(not(all(any(unix, windows), not(any(target_os = "ios", target_os = "android")))))]
    Err(Error::Invalid("platform".into(), "no supported OS credential store".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use only the library's in-memory mock. Never read, overwrite or delete a
    // real resident's saved credentials while running tests.
    #[test]
    fn core_entries_preserve_service_user_and_secret_lifecycle() {
        keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
        let first = Entry::new("Amante Viewer", "grid|resident").unwrap();
        assert!(matches!(first.get_password(), Err(Error::NoEntry)));
        first.set_password("test-only-value").unwrap();
        assert_eq!(first.get_password().unwrap(), "test-only-value");
        let reopened = Entry::new("Amante Viewer", "grid|resident").unwrap();
        assert_eq!(reopened.get_password().unwrap(), "test-only-value");
        let mfa = Entry::new("Amante Viewer MFA", "grid|resident").unwrap();
        let another = Entry::new("Amante Viewer", "grid|another").unwrap();
        assert!(matches!(mfa.get_password(), Err(Error::NoEntry)));
        assert!(matches!(another.get_password(), Err(Error::NoEntry)));
        reopened.delete_credential().unwrap();
        assert!(matches!(first.get_password(), Err(Error::NoEntry)));
    }
}
