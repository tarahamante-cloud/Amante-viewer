//! Plugin process parent (LLPluginProcessParent): listens on a loopback TCP
//! port, launches SLPlugin with that port, tells it which plugin DLL to load
//! and exchanges `\0`-terminated LLSD XML messages with it. Internal messages
//! (hello, load_plugin_response, heartbeat, shm_*) are handled here; the
//! others are queued for the media class.
//!
//! Port of indra/llplugin/llpluginprocessparent.cpp (Copyright (C) Linden
//! Research, Inc., originally LGPL 2.1). Everything runs from `idle()` with non-blocking
//! sockets, like the parent's idle loop without its optional pump thread.

use crate::message::PluginMessage;
use crate::shm::SharedMemory;
use std::collections::{HashMap, VecDeque};
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub const CLASS_INTERNAL: &str = "internal";

/// mPluginLaunchTimeout / mPluginLockupTimeout.
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(60);
const LOCKUP_TIMEOUT: Duration = Duration::from_secs(15);
/// GOODBYE_SECONDS of the child is 12 s: give it a little more.
const EXIT_TIMEOUT: Duration = Duration::from_secs(14);
pub(crate) const MAX_FRAME: usize = 1024 * 1024;
const MAX_QUEUED_BYTES: usize = 4 * MAX_FRAME;
const MAX_MESSAGES: usize = 256;
const IO_BUDGET: usize = 256 * 1024;
const FRAME_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_SHM_SEGMENT: usize = 64 * 1024 * 1024;
const MAX_SHM_TOTAL: usize = 128 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcState {
    /// Listening, process launched, waiting for its connection.
    Launched,
    /// Connected, waiting for "hello".
    Connected,
    /// load_plugin sent.
    Loading,
    Running,
    /// shutdown_plugin sent.
    Exiting,
    Done,
    Error(String),
}

pub struct PluginProcess {
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    child: Option<Child>,
    state: ProcState,
    input: Vec<u8>,
    input_started: Option<Instant>,
    output: Vec<u8>,
    heartbeat: Instant,
    plugin_file: String,
    plugin_dir: String,
    shm: HashMap<String, SharedMemory>,
    received: VecDeque<PluginMessage>,
    received_bytes: usize,
    sleep_time: f64,
    exit_deadline: Option<Instant>,
    pub plugin_version: String,
    pub cpu_usage: f64,
}

impl PluginProcess {
    /// The system id of the plugin process while it runs.
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(Child::id)
    }

    /// Bind the listening socket and launch `launcher` (SLPlugin) with the
    /// port as its only argument, in `plugin_dir`, with `env` added to its
    /// environment.
    pub fn launch(launcher: &Path, plugin_dir: &Path, plugin_file: &Path, env: &[(String, String)]) -> std::io::Result<PluginProcess> {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let mut cmd = Command::new(launcher);
        cmd.arg(port.to_string()).current_dir(plugin_dir);
        cmd.envs(env.iter().map(|(name, value)| (name, value)));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        // Linux: CEF's host process (dullahan_host, started by CEF with this
        // environment) has no rpath that finds libcef.so; see
        // `plugin_library_path`.
        #[cfg(target_os = "linux")]
        if let Some(dir) = launcher.parent() {
            let inherited = std::env::var_os("LD_LIBRARY_PATH");
            if let Ok(joined) = std::env::join_paths(plugin_library_path(dir, inherited.as_deref())) {
                cmd.env("LD_LIBRARY_PATH", joined);
            }
        }
        let child = cmd.spawn()?;
        log::info!(
            "media: launched {} (port {port}) for {}",
            launcher.display(),
            plugin_file
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_default()
        );
        Ok(PluginProcess {
            listener: Some(listener),
            stream: None,
            child: Some(child),
            state: ProcState::Launched,
            input: Vec::new(),
            input_started: None,
            output: Vec::new(),
            heartbeat: Instant::now() + LAUNCH_TIMEOUT,
            plugin_file: plugin_file.to_string_lossy().into_owned(),
            plugin_dir: plugin_dir.to_string_lossy().into_owned(),
            shm: HashMap::new(),
            received: VecDeque::new(),
            received_bytes: 0,
            sleep_time: 0.01,
            exit_deadline: None,
            plugin_version: String::new(),
            cpu_usage: 0.0,
        })
    }

    pub fn state(&self) -> &ProcState {
        &self.state
    }

    pub fn is_running(&self) -> bool {
        self.state == ProcState::Running
    }

    pub fn is_done(&self) -> bool {
        matches!(self.state, ProcState::Done | ProcState::Error(_))
    }

    pub fn error(&self) -> Option<&str> {
        match &self.state {
            ProcState::Error(e) => Some(e),
            _ => None,
        }
    }

    pub(crate) fn fail(&mut self, why: String) {
        if !matches!(self.state, ProcState::Error(_)) {
            log::warn!("media: plugin {} failed: {why}", self.plugin_file);
        }
        self.state = ProcState::Error(why);
        self.stream = None;
        self.listener = None;
        self.input.clear();
        self.output.clear();
        self.received.clear();
        self.received_bytes = 0;
        self.shm.clear();
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    /// Queue a message on the socket (sent by the next `idle`).
    pub fn send(&mut self, m: &PluginMessage) {
        if self.stream.is_none() {
            return;
        }
        log::trace!("media → plugin: {}:{}", m.class, m.name);
        let raw = m.generate();
        if raw.len() > MAX_FRAME || self.output.len() + raw.len() + 1 > MAX_QUEUED_BYTES {
            self.fail("outgoing media queue limit exceeded".into());
            return;
        }
        self.output.extend_from_slice(raw.as_bytes());
        self.output.push(0);
    }

    /// setSleepTime: how long the plugin sleeps between idles.
    pub fn set_sleep_time(&mut self, t: f64, force: bool) {
        if force || (t - self.sleep_time).abs() > 1e-6 {
            self.sleep_time = t;
            let m = PluginMessage::new(CLASS_INTERNAL, "sleep_time").with("time", t);
            self.send(&m);
        }
    }

    /// Messages for the media class, in arrival order.
    pub fn take_messages(&mut self) -> VecDeque<PluginMessage> {
        self.received_bytes = 0;
        std::mem::take(&mut self.received)
    }

    pub fn add_shared_memory(&mut self, size: usize) -> Option<String> {
        if size == 0
            || size > MAX_SHM_SEGMENT
            || self.shm.len() >= 4
            || self.shm.values().map(SharedMemory::size).sum::<usize>() + size > MAX_SHM_TOTAL
        {
            self.fail("shared media memory limit exceeded".into());
            return None;
        }
        let seg = SharedMemory::create(size)?;
        let name = seg.name().to_string();
        let m = PluginMessage::new(CLASS_INTERNAL, "shm_add")
            .with("name", name.as_str())
            .with("size", size as i32);
        self.shm.insert(name.clone(), seg);
        self.send(&m);
        Some(name)
    }

    /// The plugin unmaps first; ours is released on shm_remove_response.
    pub fn remove_shared_memory(&mut self, name: &str) {
        if self.shm.contains_key(name) {
            let m = PluginMessage::new(CLASS_INTERNAL, "shm_remove").with("name", name);
            self.send(&m);
        }
    }

    pub fn shared_memory(&self, name: &str) -> Option<&SharedMemory> {
        self.shm.get(name)
    }

    /// requestShutdown: ask the plugin to exit; killed if it lingers.
    pub fn shutdown(&mut self) {
        match self.state {
            ProcState::Running | ProcState::Loading | ProcState::Connected => {
                let m = PluginMessage::new(CLASS_INTERNAL, "shutdown_plugin");
                self.send(&m);
                if self.is_done() {
                    return;
                }
                let _ = self.pump_output();
                self.state = ProcState::Exiting;
                self.exit_deadline = Some(Instant::now() + EXIT_TIMEOUT);
            }
            ProcState::Launched => self.fail("shut down before connecting".into()),
            _ => {}
        }
    }

    fn child_exited(&mut self) -> bool {
        match self.child.as_mut().map(|c| c.try_wait()) {
            Some(Ok(Some(status))) => {
                log::info!("media: plugin process exited ({status})");
                self.child = None;
                true
            }
            Some(Ok(None)) => false,
            Some(Err(_)) | None => true,
        }
    }

    pub fn idle(&mut self) {
        match self.state {
            ProcState::Done | ProcState::Error(_) => return,
            ProcState::Exiting => {
                let _ = self.pump_output();
                let _ = self.pump_input();
                if self.child_exited() {
                    self.state = ProcState::Done;
                    self.stream = None;
                } else if self.exit_deadline.is_some_and(|d| Instant::now() > d) {
                    self.fail("did not exit in time".into());
                }
                return;
            }
            _ => {}
        }
        if self.state == ProcState::Launched {
            match self.listener.as_ref().map(|l| l.accept()) {
                Some(Ok((stream, _))) => {
                    // The child remains owned/live, preventing PID reuse.
                    // Reject competitors without consuming the listener or
                    // changing the original launch deadline. No wire changes.
                    let authorized = self
                        .pid()
                        .is_some_and(|pid| amante_local_ipc::peer::owns_connection(&stream, pid).unwrap_or(false));
                    if authorized {
                        if let Err(e) = stream.set_nonblocking(true).and_then(|()| stream.set_nodelay(true)) {
                            return self.fail(format!("socket setup: {e}"));
                        }
                        self.stream = Some(stream);
                        self.listener = None;
                        self.state = ProcState::Connected;
                    } else {
                        log::debug!("media: rejected unauthorized local peer");
                    }
                }
                Some(Err(e)) if e.kind() == ErrorKind::WouldBlock => {}
                Some(Err(e)) => return self.fail(format!("accept: {e}")),
                None => {}
            }
        }
        if self.stream.is_some() {
            if let Err(e) = self.pump_output() {
                return self.fail(format!("send: {e}"));
            }
            if let Err(e) = self.pump_input() {
                return self.fail(format!("receive: {e}"));
            }
        }
        // pluginLockedUpOrQuit
        if self.child_exited() {
            return self.fail("plugin process quit".into());
        }
        if Instant::now() > self.heartbeat {
            self.fail(
                if self.state == ProcState::Running {
                    "plugin locked up"
                } else {
                    "plugin launch timed out"
                }
                .into(),
            );
        }
    }

    fn pump_output(&mut self) -> std::io::Result<()> {
        let Some(stream) = self.stream.as_mut() else {
            return Ok(());
        };
        let mut budget = IO_BUDGET;
        while !self.output.is_empty() && budget > 0 {
            match stream.write(&self.output[..self.output.len().min(budget)]) {
                Ok(0) => return Err(ErrorKind::WriteZero.into()),
                Ok(n) => {
                    budget -= n;
                    self.output.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn pump_input(&mut self) -> std::io::Result<()> {
        let mut buf = [0u8; 16384];
        let mut budget = IO_BUDGET;
        let mut messages = 0;
        loop {
            while let Some(end) = self.input.iter().position(|&b| b == 0) {
                if end > MAX_FRAME {
                    return Err(ErrorKind::InvalidData.into());
                }
                if messages == 64 {
                    return Ok(());
                }
                let raw: Vec<u8> = self.input.drain(..=end).collect();
                let m = PluginMessage::parse(&raw[..end]).ok_or(ErrorKind::InvalidData)?;
                if m.class != CLASS_INTERNAL {
                    if self.received.len() >= MAX_MESSAGES || self.received_bytes + raw.len() > MAX_QUEUED_BYTES {
                        return Err(ErrorKind::OutOfMemory.into());
                    }
                    self.received_bytes += raw.len();
                }
                self.receive(m);
                if self.is_done() {
                    return Ok(());
                }
                messages += 1;
                self.input_started = (!self.input.is_empty()).then(Instant::now);
            }
            if self.input.len() > MAX_FRAME {
                return Err(ErrorKind::InvalidData.into());
            }
            if self.input_started.is_some_and(|start| start.elapsed() > FRAME_TIMEOUT) {
                return Err(ErrorKind::TimedOut.into());
            }
            if budget == 0 {
                return Ok(());
            }
            let Some(stream) = self.stream.as_mut() else {
                return Ok(());
            };
            let limit = buf.len().min(budget).min(MAX_FRAME + 1 - self.input.len());
            match stream.read(&mut buf[..limit]) {
                Ok(0) => {
                    if self.state == ProcState::Exiting {
                        self.stream = None;
                        return Ok(());
                    }
                    return Err(ErrorKind::ConnectionAborted.into());
                }
                Ok(n) => {
                    if self.input.is_empty() {
                        self.input_started = Some(Instant::now());
                    }
                    self.input.extend_from_slice(&buf[..n]);
                    budget -= n;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn receive(&mut self, m: PluginMessage) {
        if m.class != CLASS_INTERNAL {
            if self.state != ProcState::Running {
                self.fail("media message before plugin ready".into());
                return;
            }
            log::trace!("media ← plugin: {}:{}", m.class, m.name);
            self.received.push_back(m);
            return;
        }
        match m.name.as_str() {
            "hello" => {
                if self.state == ProcState::Connected {
                    let lp = PluginMessage::new(CLASS_INTERNAL, "load_plugin")
                        .with("file", self.plugin_file.as_str())
                        .with("dir", self.plugin_dir.as_str());
                    self.send(&lp);
                    if self.is_done() {
                        return;
                    }
                    self.state = ProcState::Loading;
                } else {
                    self.fail("hello in wrong state".into());
                }
            }
            "load_plugin_response" => {
                if self.state == ProcState::Loading {
                    self.plugin_version = m.string("plugin_version");
                    log::info!("media: plugin version {}", self.plugin_version);
                    let t = self.sleep_time;
                    self.set_sleep_time(t, true);
                    if self.is_done() {
                        return;
                    }
                    self.state = ProcState::Running;
                    self.heartbeat = Instant::now() + LOCKUP_TIMEOUT;
                } else {
                    self.fail("load_plugin_response in wrong state".into());
                }
            }
            "heartbeat" => {
                if self.state == ProcState::Running {
                    self.heartbeat = Instant::now() + LOCKUP_TIMEOUT;
                    self.cpu_usage = m.real("cpu_usage");
                }
            }
            "shm_add_response" => {}
            "shm_remove_response" => {
                let name = m.string("name");
                self.shm.remove(&name);
            }
            other => log::debug!("media: unknown internal message {other}"),
        }
    }
}

impl Drop for PluginProcess {
    fn drop(&mut self) {
        if matches!(self.state, ProcState::Running | ProcState::Loading | ProcState::Connected) {
            self.shutdown();
        }
        // give the plugin time to shut CEF down cleanly, then make sure it is gone
        if let Some(mut child) = self.child.take() {
            let stream = self.stream.take();
            let _ = std::thread::Builder::new().name("media plugin exit".into()).spawn(move || {
                let deadline = Instant::now() + EXIT_TIMEOUT;
                while Instant::now() < deadline {
                    if let Ok(Some(_)) = child.try_wait() {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                drop(stream);
                let _ = child.kill();
                let _ = child.wait();
            });
        }
    }
}

/// `LD_LIBRARY_PATH` of the plugin process on Linux: SLPlugin's folder
/// (Aurora's pack keeps libcef.so there, in `llplugin/`), then the `lib`
/// folder beside it when there is one (a Legacy Viewer install chosen as the
/// plugins folder: `bin/SLPlugin`, `lib/libcef.so`; its wrapper.sh puts
/// `lib` first in LD_LIBRARY_PATH, indra/newview/linux_tools/wrapper.sh),
/// then the inherited value. The folder is resolved first, as wrapper.sh
/// resolves its own path with `readlink -f`: `lib` sits beside the real
/// `bin`, not beside a link to it. Empty entries are dropped: the loader
/// reads one as the current folder, which wrapper.sh adds when
/// LD_LIBRARY_PATH was unset or empty.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn plugin_library_path(launcher_dir: &Path, inherited: Option<&std::ffi::OsStr>) -> Vec<PathBuf> {
    let dir = launcher_dir.canonicalize().unwrap_or_else(|_| launcher_dir.to_path_buf());
    let mut paths = vec![dir.clone()];
    if let Some(lib) = dir.parent().map(|p| p.join("lib")).filter(|lib| lib.is_dir()) {
        paths.push(lib);
    }
    if let Some(old) = inherited {
        paths.extend(std::env::split_paths(old).filter(|p| !p.as_os_str().is_empty()));
    }
    paths
}

/// File names of the launcher: `SLPlugin.exe` on Windows, `SLPlugin`
/// elsewhere.
const LAUNCHERS: &[&str] = if cfg!(windows) {
    &["SLPlugin.exe", "slplugin.exe"]
} else {
    &["SLPlugin"]
};

/// Where SLPlugin and the media plugin libraries live.
#[derive(Debug, Clone)]
pub struct PluginPaths {
    pub launcher: PathBuf,
    pub plugin_dir: PathBuf,
}

impl PluginPaths {
    /// Path of a plugin library (`media_plugin_cef`, `media_plugin_libvlc`,
    /// `media_plugin_gstreamer`): `<name>.dll` on Windows, `<name>.dylib` on
    /// macOS (LLDir_Mac::getLLPluginFilename), `lib<name>.so` elsewhere.
    pub fn plugin_file(&self, basename: &str) -> PathBuf {
        let file = if cfg!(windows) {
            format!("{basename}.dll")
        } else if cfg!(target_os = "macos") {
            format!("{basename}.dylib")
        } else {
            format!("lib{basename}.so")
        };
        self.plugin_dir.join(file)
    }

    pub fn has_plugin(&self, basename: &str) -> bool {
        self.plugin_file(basename).is_file()
    }

    /// A folder holding `llplugin\` and SLPlugin (either inside `llplugin\`,
    /// as in Aurora's pack, or next to it, as in a Firestorm / SL install);
    /// on macOS, a folder holding SLPlugin.app ([`Self::bundle_in`]).
    pub fn in_dir(dir: &Path) -> Option<PluginPaths> {
        if cfg!(target_os = "macos") {
            return Self::bundle_in(dir);
        }
        let plugin_dir = dir.join("llplugin");
        if !plugin_dir.is_dir() {
            return None;
        }
        for exe in LAUNCHERS {
            for base in [dir, plugin_dir.as_path()] {
                let launcher = base.join(exe);
                if launcher.is_file() {
                    return Some(PluginPaths {
                        launcher,
                        plugin_dir: plugin_dir.clone(),
                    });
                }
            }
        }
        None
    }

    /// macOS: SLPlugin.app laid out as in the Legacy Viewer's application
    /// (indra/llfilesystem/lldir_mac.cpp: the launcher in its Contents/MacOS,
    /// the plugins, CEF's framework and helpers and LibVLC in its
    /// Contents/Frameworks, the folder SLPlugin runs in). Found in
    /// `llplugin/` (Aurora's pack), in the folder itself, or in the
    /// Contents/Resources of an application chosen as the folder (an
    /// installed Legacy Viewer's .app).
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    fn bundle_in(dir: &Path) -> Option<PluginPaths> {
        [dir.join("llplugin"), dir.to_path_buf(), dir.join("Contents").join("Resources")]
            .into_iter()
            .map(|base| base.join("SLPlugin.app"))
            .find_map(|app| {
                let contents = app.join("Contents");
                let launcher = contents.join("MacOS").join("SLPlugin");
                launcher.is_file().then(|| PluginPaths {
                    launcher,
                    plugin_dir: contents.join("Frameworks"),
                })
            })
    }

    /// Search order: the folder chosen by the user, the viewer's own folder,
    /// then Aurora's media plugins pack. The plugins of another program are
    /// only used when the user names its folder: an installed Firestorm or
    /// Second Life viewer is not looked for.
    ///
    /// A relative folder is made absolute against the viewer's current folder
    /// first: SLPlugin is started in `plugin_dir`, so a relative launcher
    /// (Unix), `LD_LIBRARY_PATH` (Linux) or plugin file and folder sent in
    /// `load_plugin` would then be looked for from there and never found.
    /// `std::path::absolute`, not `canonicalize`, so that Windows paths keep
    /// no `\\?\` prefix and links stay as the user chose them. A folder that
    /// cannot be made absolute (current folder deleted) is skipped.
    pub fn discover(explicit: Option<&Path>, pack: Option<&Path>) -> Option<PluginPaths> {
        let own = std::env::current_exe().ok();
        let own = own.as_deref().and_then(Path::parent);
        [explicit, own, pack]
            .into_iter()
            .flatten()
            .filter_map(|dir| std::path::absolute(dir).ok())
            .find_map(|dir| PluginPaths::in_dir(&dir))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn fixture() -> PluginProcess {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        PluginProcess {
            listener: Some(listener),
            stream: None,
            child: None,
            state: ProcState::Launched,
            input: Vec::new(),
            input_started: None,
            output: Vec::new(),
            heartbeat: Instant::now() + LAUNCH_TIMEOUT,
            plugin_file: "synthetic-plugin".into(),
            plugin_dir: ".".into(),
            shm: HashMap::new(),
            received: VecDeque::new(),
            received_bytes: 0,
            sleep_time: 0.01,
            exit_deadline: None,
            plugin_version: String::new(),
            cpu_usage: 0.0,
        }
    }

    fn with_socket() -> (PluginProcess, TcpStream) {
        let mut process = fixture();
        let client = TcpStream::connect(process.listener.as_ref().unwrap().local_addr().unwrap()).unwrap();
        let (stream, _) = process.listener.take().unwrap().accept().unwrap();
        stream.set_nonblocking(true).unwrap();
        process.stream = Some(stream);
        process.state = ProcState::Running;
        (process, client)
    }

    pub(crate) fn release_child(process: &mut PluginProcess) {
        process.child.as_mut().unwrap().stdin.as_mut().unwrap().write_all(b"x").unwrap();
    }

    pub(crate) fn launched_fixture() -> PluginProcess {
        use std::process::Stdio;
        let mut process = fixture();
        let port = process.listener.as_ref().unwrap().local_addr().unwrap().port();
        process.child = Some(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "process::tests::synthetic_plugin_peer", "--ignored", "--nocapture"])
                .env("AMANTE_TEST_PLUGIN_PORT", port.to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()
                .unwrap(),
        );
        process
    }

    fn until(process: &mut PluginProcess, expected: ProcState) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while process.state != expected && !process.is_done() && Instant::now() < deadline {
            process.idle();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(process.state, expected);
    }

    #[test]
    #[ignore = "subprocess fixture, invoked by the synthetic peer tests"]
    fn synthetic_plugin_peer() {
        let Some(port) = std::env::var_os("AMANTE_TEST_PLUGIN_PORT") else {
            return;
        };
        std::io::stdin().read_exact(&mut [0]).unwrap();
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port.to_string_lossy().parse::<u16>().unwrap())).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream
            .write_all(format!("{}\0", PluginMessage::new(CLASS_INTERNAL, "hello").generate()).as_bytes())
            .unwrap();
        let mut input = Vec::new();
        loop {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            if byte[0] != 0 {
                input.push(byte[0]);
                continue;
            }
            let mut m = PluginMessage::parse(&input).unwrap();
            input.clear();
            match m.name.as_str() {
                "load_plugin" => stream
                    .write_all(
                        format!(
                            "{}\0",
                            PluginMessage::new(CLASS_INTERNAL, "load_plugin_response")
                                .with("plugin_version", "synthetic")
                                .generate()
                        )
                        .as_bytes(),
                    )
                    .unwrap(),
                "shutdown_plugin" => return,
                "size_change" => {
                    m.name = "size_change_response".into();
                    stream.write_all(format!("{}\0", m.generate()).as_bytes()).unwrap();
                }
                _ => {}
            }
        }
    }

    #[test]
    #[cfg(any(windows, target_os = "linux"))]
    fn amante_security_competitor_cannot_claim_child_connection_and_clean_shutdown() {
        let mut process = launched_fixture();
        let mut attacker = TcpStream::connect(process.listener.as_ref().unwrap().local_addr().unwrap()).unwrap();
        attacker
            .write_all(format!("{}\0", PluginMessage::new(CLASS_INTERNAL, "hello").generate()).as_bytes())
            .unwrap();
        process.idle();
        assert_eq!(process.state, ProcState::Launched);
        assert!(process.listener.is_some());
        assert!(process.output.is_empty());
        process.child.as_mut().unwrap().stdin.as_mut().unwrap().write_all(b"x").unwrap();
        until(&mut process, ProcState::Running);
        assert_eq!(process.plugin_version, "synthetic");
        process.shutdown();
        until(&mut process, ProcState::Done);
    }

    #[test]
    fn amante_security_unpaired_traffic_cannot_extend_launch_timeout_and_relaunch_works() {
        let mut process = launched_fixture();
        let _attacker = TcpStream::connect(process.listener.as_ref().unwrap().local_addr().unwrap()).unwrap();
        process.heartbeat = Instant::now() - Duration::from_secs(1);
        process.idle();
        assert!(process.is_done());
        assert!(process.child.is_none());
        assert!(process.listener.is_none());
        #[cfg(any(windows, target_os = "linux"))]
        {
            let mut replacement = launched_fixture();
            replacement.child.as_mut().unwrap().stdin.as_mut().unwrap().write_all(b"x").unwrap();
            until(&mut replacement, ProcState::Running);
            replacement.shutdown();
            until(&mut replacement, ProcState::Done);
        }
    }

    #[test]
    fn amante_security_media_frame_queue_and_partial_deadline_limits() {
        let (mut process, _client) = with_socket();
        process.input = vec![b'x'; MAX_FRAME + 1];
        assert_eq!(process.pump_input().unwrap_err().kind(), ErrorKind::InvalidData);
        process.input = vec![b'x'];
        process.input_started = Some(Instant::now() - FRAME_TIMEOUT - Duration::from_secs(1));
        assert_eq!(process.pump_input().unwrap_err().kind(), ErrorKind::TimedOut);
        process.input = b"<llsd>\0".to_vec();
        assert_eq!(process.pump_input().unwrap_err().kind(), ErrorKind::InvalidData);
        process.input = format!("{}\0", PluginMessage::new("media", "event").generate()).into_bytes();
        process.received_bytes = MAX_QUEUED_BYTES;
        assert_eq!(process.pump_input().unwrap_err().kind(), ErrorKind::OutOfMemory);
        process.send(&PluginMessage::new("media", "too-large").with("value", "x".repeat(MAX_FRAME)));
        assert!(process.is_done());
        assert!(process.input.is_empty() && process.output.is_empty() && process.received.is_empty());
    }

    #[test]
    fn amante_security_media_flood_has_per_idle_work_and_output_bounds() {
        let (mut process, mut client) = with_socket();
        let message = format!("{}\0", PluginMessage::new("media", "event").generate());
        client.write_all(message.repeat(200).as_bytes()).unwrap();
        process.pump_input().unwrap();
        assert_eq!(process.received.len(), 64);
        process.pump_input().unwrap();
        assert_eq!(process.received.len(), 128);
        process.output = vec![b'x'; MAX_QUEUED_BYTES];
        process.send(&PluginMessage::new("media", "event"));
        assert!(process.is_done());
    }

    #[test]
    fn amante_security_fragmented_frames_heartbeat_and_memory_limits() {
        let (mut process, mut client) = with_socket();
        let message = PluginMessage::new("media", "event").generate();
        client.write_all(&message.as_bytes()[..10]).unwrap();
        process.pump_input().unwrap();
        assert!(process.received.is_empty());
        client.write_all(&message.as_bytes()[10..]).unwrap();
        client.write_all(&[0]).unwrap();
        process.pump_input().unwrap();
        assert_eq!(process.take_messages().len(), 1);
        assert_eq!(process.received_bytes, 0);
        process.state = ProcState::Loading;
        let deadline = process.heartbeat;
        process.receive(PluginMessage::new(CLASS_INTERNAL, "heartbeat"));
        assert_eq!(process.heartbeat, deadline);
        assert!(process.add_shared_memory(MAX_SHM_SEGMENT + 1).is_none());
        assert!(process.is_done());
        let (mut process, _client) = with_socket();
        for _ in 0..4 {
            assert!(process.add_shared_memory(1).is_some());
        }
        assert!(process.add_shared_memory(1).is_none());
        assert!(process.shm.is_empty());
        let (mut process, _client) = with_socket();
        assert!(process.add_shared_memory(MAX_SHM_SEGMENT).is_some());
        assert!(process.add_shared_memory(MAX_SHM_SEGMENT).is_some());
        assert!(process.add_shared_memory(1).is_none());
        assert!(process.shm.is_empty());
    }

    #[test]
    fn amante_security_running_child_lockup_releases_buffers_and_mappings() {
        let mut process = launched_fixture();
        process.state = ProcState::Running;
        process.heartbeat = Instant::now() - LOCKUP_TIMEOUT;
        process.input.extend_from_slice(b"partial");
        process.output.extend_from_slice(b"pending");
        process.idle();
        assert!(process.error().is_some_and(|error| error.contains("locked up")));
        assert!(process.child.is_none());
        assert!(process.input.is_empty() && process.output.is_empty() && process.shm.is_empty());
    }

    fn folder(root: &Path, name: &str, launcher_inside: bool) -> PathBuf {
        let dir = root.join(name);
        std::fs::create_dir_all(dir.join("llplugin")).expect("fixture");
        let launcher = if launcher_inside { dir.join("llplugin") } else { dir.clone() }.join(LAUNCHERS[0]);
        std::fs::write(launcher, b"").expect("fixture");
        dir
    }

    // (macOS looks for SLPlugin.app instead: the test below)
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn the_chosen_folder_comes_before_the_pack_and_nothing_else_is_searched() {
        let root = std::env::temp_dir().join(format!("aurora-media-paths-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let chosen = folder(&root, "chosen", false);
        let pack = folder(&root, "pack", true);
        let empty = root.join("empty");
        std::fs::create_dir_all(&empty).expect("fixture");

        let found = PluginPaths::discover(Some(&chosen), Some(&pack)).expect("chosen folder");
        assert_eq!(found.launcher, chosen.join(LAUNCHERS[0]));
        // a folder without plugins falls through to the pack
        let found = PluginPaths::discover(Some(&empty), Some(&pack)).expect("pack");
        assert_eq!(found.launcher, pack.join("llplugin").join(LAUNCHERS[0]));
        assert_eq!(found.plugin_dir, pack.join("llplugin"));
        assert_eq!(
            found.plugin_file("media_plugin_cef"),
            pack.join("llplugin").join(if cfg!(windows) {
                "media_plugin_cef.dll"
            } else {
                "libmedia_plugin_cef.so"
            })
        );
        // no pack: nothing, even on a PC where another viewer is installed
        assert!(PluginPaths::discover(Some(&empty), None).is_none());
        assert!(PluginPaths::discover(None, None).is_none());
        std::fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn plugin_library_path_adds_the_lib_folder_beside_slplugin_and_drops_empty_entries() {
        let root = std::env::temp_dir().join(format!("aurora-media-ldpath-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        // a Legacy Viewer install: bin/SLPlugin, lib/libcef.so
        let bin = root.join("legacy").join("bin");
        let lib = root.join("legacy").join("lib");
        std::fs::create_dir_all(&bin).expect("fixture");
        std::fs::create_dir_all(&lib).expect("fixture");
        // Aurora's pack: llplugin/SLPlugin, nothing beside llplugin
        let pack = root.join("pack").join("llplugin");
        std::fs::create_dir_all(&pack).expect("fixture");
        // the paths come back resolved (macOS' temp folder is a link,
        // Windows' gets its `\\?\` prefix)
        let (bin, lib, pack) = (
            bin.canonicalize().expect("fixture"),
            lib.canonicalize().expect("fixture"),
            pack.canonicalize().expect("fixture"),
        );
        let sep = if cfg!(windows) { ";" } else { ":" };

        assert_eq!(plugin_library_path(&bin, None), vec![bin.clone(), lib.clone()]);
        assert_eq!(plugin_library_path(&pack, None), vec![pack.clone()]);
        let inherited = std::ffi::OsString::from(format!("/usr/local/lib{sep}{sep}/opt/x"));
        assert_eq!(
            plugin_library_path(&pack, Some(&inherited)),
            vec![pack.clone(), PathBuf::from("/usr/local/lib"), PathBuf::from("/opt/x")]
        );
        // set but empty: no trailing empty entry (the current folder)
        assert_eq!(
            plugin_library_path(&bin, Some(std::ffi::OsStr::new(""))),
            vec![bin.clone(), lib.clone()]
        );
        // a link to the install's bin: lib is found beside the real folder
        #[cfg(unix)]
        {
            let link = root.join("link-to-bin");
            std::os::unix::fs::symlink(&bin, &link).expect("fixture");
            assert_eq!(plugin_library_path(&link, None), vec![bin.clone(), lib]);
        }
        std::fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn macos_finds_slplugin_app_in_the_pack_a_folder_or_an_application() {
        let root = std::env::temp_dir().join(format!("aurora-media-bundle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let bundle = |base: &Path| {
            let macos = base.join("SLPlugin.app").join("Contents").join("MacOS");
            std::fs::create_dir_all(&macos).expect("fixture");
            std::fs::write(macos.join("SLPlugin"), b"").expect("fixture");
            base.join("SLPlugin.app").join("Contents")
        };
        let pack = root.join("pack");
        let pack_contents = bundle(&pack.join("llplugin"));
        let legacy = root.join("Legacy.app");
        let legacy_contents = bundle(&legacy.join("Contents").join("Resources"));
        let found = PluginPaths::bundle_in(&pack).expect("pack");
        assert_eq!(found.launcher, pack_contents.join("MacOS").join("SLPlugin"));
        assert_eq!(found.plugin_dir, pack_contents.join("Frameworks"));
        let found = PluginPaths::bundle_in(&legacy).expect("an application");
        assert_eq!(found.plugin_dir, legacy_contents.join("Frameworks"));
        assert!(PluginPaths::bundle_in(&root).is_none(), "nothing directly in the root");
        let _ = std::fs::remove_dir_all(&root);
    }

    // (macOS looks for SLPlugin.app instead)
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn a_relative_folder_gives_an_absolute_launcher_and_plugin_dir() {
        // relative to the test's current folder, as a folder typed in the
        // preferences is relative to the viewer's
        let rel = PathBuf::from(format!("aurora-media-relative-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&rel);
        folder(Path::new("."), rel.to_str().expect("ascii name"), false);
        let found = PluginPaths::discover(Some(&rel), None).expect("relative folder");
        let abs = std::env::current_dir().expect("current folder").join(&rel);
        std::fs::remove_dir_all(&rel).expect("cleanup");
        assert!(found.launcher.is_absolute(), "{}", found.launcher.display());
        assert_eq!(found.launcher, abs.join(LAUNCHERS[0]));
        assert_eq!(found.plugin_dir, abs.join("llplugin"));
    }
}
