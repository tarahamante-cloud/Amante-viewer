//! User-requested Windows desktop delivery. No renderer or settings migration.
#[cfg(windows)]
fn run(remove: bool) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let script = include_str!("../../../scripts/desktop-shortcut.ps1");
    // Values are environment variables, never interpolated into PowerShell.
    let command = format!(
        "& {{ {script} }} -Executable $env:AMANTE_SHORTCUT_EXE {}",
        if remove { "-Remove" } else { "" }
    );
    let status = std::process::Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", &command])
        .env("AMANTE_SHORTCUT_EXE", exe)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("desktop shortcut command failed: {status}"))
    }
}

pub fn start() {
    #[cfg(windows)]
    if !cfg!(test) && std::env::var_os("AURORA_DEMO").is_none() && aurora_updater::test_environment().is_none() {
        let marker = crate::settings::config_dir().join("amante-desktop-shortcut");
        let exe = std::env::current_exe().ok();
        let identity = exe.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        if std::fs::read_to_string(&marker).ok().as_deref() == Some(&identity) {
            return;
        }
        let result = std::thread::Builder::new().name("amante-desktop-shortcut".into()).spawn(move || {
            if let Err(error) = run(false) {
                log::warn!("Desktop shortcut could not be created: {error}");
                return;
            }
            if let Some(parent) = marker.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(error) = std::fs::write(marker, identity) {
                log::warn!("Desktop shortcut marker could not be saved: {error}");
            }
        });
        if let Err(error) = result {
            log::warn!("Desktop shortcut worker could not start: {error}");
        }
    }
}

pub fn remove() -> Result<(), String> {
    #[cfg(windows)]
    if !cfg!(test) && aurora_updater::test_environment().is_none() {
        run(true)?;
        let marker = crate::settings::config_dir().join("amante-desktop-shortcut");
        match std::fs::remove_file(marker) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

pub fn registration() -> Option<String> {
    if cfg!(windows) && crate::settings::config_dir().join("amante-desktop-shortcut").exists() {
        Some("Windows desktop: Amante Viewer Alpha.lnk".to_owned())
    } else {
        None
    }
}
