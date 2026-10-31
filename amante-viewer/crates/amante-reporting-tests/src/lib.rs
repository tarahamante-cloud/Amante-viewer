//! Offline tests of the real viewer reporting code, without a desktop or grid.
//! Requests in module tests go only to loopback fixtures with dummy tokens.

#[cfg(test)]
#[allow(dead_code)]
#[path = "../../aurora-viewer/src/bug_report/model.rs"]
mod bug_report;

#[cfg(test)]
#[allow(dead_code)]
#[path = "../../aurora-viewer/src/bug_report/github.rs"]
mod bug_report_github;

#[cfg(test)]
#[allow(dead_code)]
#[path = "../../aurora-viewer/src/crash/github.rs"]
mod crash_github;

#[cfg(test)]
mod wiring_tests {
    // Source-level regression guard, not a desktop UI test. The native
    // compile check additionally verifies these modules together in App.
    fn active_source(source: &str) -> String {
        source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn native_report_actions_keep_the_form_in_the_active_build() {
        let main = active_source(include_str!("../../aurora-viewer/src/main.rs"));
        let ui = active_source(include_str!("../../aurora-viewer/src/ui/mod.rs"));
        let app = active_source(include_str!("../../aurora-viewer/src/app.rs"));
        assert!(main.contains("mod bug_report;"));
        assert!(ui.contains("pub mod bug_report;"));
        for required in [
            "mod bug_report;",
            "bug_report: ui::bug_report::BugReportUi,",
            "bug_report: Default::default(),",
            "LoginAction::ReportBug => self.request_bug_report(),",
            "BarAction::ReportBug => self.request_bug_report(),",
            "fn request_bug_report(&mut self) { self.bug_report.requested = true;",
            "if self.bug_report.requested { self.open_bug_report(renderer); }",
            "self.bug_report .show(&ctx, &p, &self.skin.icons, matches!(self.screen, Screen::World));",
        ] {
            assert!(app.contains(required), "native report wiring missing: {required}");
        }
        assert!(!app.contains("self.open_issue_chooser()"));
    }
}
