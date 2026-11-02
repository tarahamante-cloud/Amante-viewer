//! Explicit native credential-store selection. Never falls back to disk files.
//!
//! Matches keyring's v1 entry API and platform stores, except that Linux uses
//! the maintained libdbus Secret Service transport instead of zbus 5.
use std::sync::LazyLock;

pub use keyring_core::{Error, Result};

static STORE: LazyLock<Result<()>> = LazyLock::new(|| {
    #[cfg(target_os = "windows")]
    let store = windows_native_keyring_store::Store::new()?;
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(target_os = "linux")]
    let store = dbus_secret_service_keyring_store::Store::new()?;
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        keyring_core::set_default_store(store);
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    Err(Error::Invalid("platform".into(), "unsupported credential store".into()))
});

pub struct Entry(keyring_core::Entry);

impl Entry {
    pub fn new(service: &str, username: &str) -> Result<Self> {
        if STORE.is_err() {
            return Err(Error::NoDefaultStore);
        }
        keyring_core::Entry::new(service, username).map(Self)
    }

    pub fn set_password(&self, password: &str) -> Result<()> {
        self.0.set_password(password)
    }

    pub fn get_password(&self) -> Result<String> {
        self.0.get_password()
    }

    pub fn delete_credential(&self) -> Result<()> {
        self.0.delete_credential()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring_core::api::CredentialStoreApi;

    #[test]
    fn facade_preserves_password_errors_unicode_and_deletion() {
        // An explicitly injected in-memory test store: no user's keychain
        // or real grid credential is opened by this unit test.
        let store = keyring_core::mock::Store::new().unwrap();
        let entry = Entry(store.build("Amante test", "resident", None).unwrap());
        assert!(matches!(entry.get_password(), Err(Error::NoEntry)));
        entry.set_password("synthetic-test-only-ä🔒").unwrap();
        assert_eq!(entry.get_password().unwrap(), "synthetic-test-only-ä🔒");
        entry.set_password("replacement-test-only").unwrap();
        assert_eq!(entry.get_password().unwrap(), "replacement-test-only");
        entry.delete_credential().unwrap();
        assert!(matches!(entry.get_password(), Err(Error::NoEntry)));
    }
}
