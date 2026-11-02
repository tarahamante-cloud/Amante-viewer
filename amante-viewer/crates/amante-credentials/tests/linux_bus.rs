//! Exercise the viewer's real Linux SLURL handoff against a private bus.
#![cfg(target_os = "linux")]
#![allow(dead_code, unused_imports)]

// The source module only needs the command-line title when choosing its name.
// No viewer, GPU, account, window or grid session is needed for these tests.
mod cli {
    pub fn title_slug() -> Option<String> {
        None
    }
}

#[path = "../../aurora-viewer/src/slurl_handoff.rs"]
mod slurl_handoff;
