//! Owner-controlled, public build configuration. No Aurora service fallback.
//! OAuth client IDs are public; tokens and signing keys do not belong here.

macro_rules! public_setting {
    ($name:ident, $env:literal) => {
        pub const $name: &str = match option_env!($env) {
            Some(value) => value,
            None => "",
        };
    };
}

public_setting!(ISSUES_URL, "AMANTE_ISSUES_URL");
public_setting!(ISSUES_API, "AMANTE_ISSUES_API");
public_setting!(GITHUB_CLIENT_ID, "AMANTE_GITHUB_CLIENT_ID");
public_setting!(RELEASES_API, "AMANTE_RELEASES_API");
public_setting!(RELEASE_PAGE, "AMANTE_RELEASE_PAGE");
public_setting!(RELEASE_DOWNLOADS, "AMANTE_RELEASE_DOWNLOADS");

/// Intentionally false until Amante has an authenticated metadata protocol,
/// pinned trust roots, rollback policy and a reviewed production release path.
/// Setting a feed URL alone must NEVER activate replacement of executable code.
pub const PRODUCTION_UPDATES_READY: bool = false;

fn reporting_destination(api: &str, issues: &str) -> bool {
    let Some(repo) = issues.strip_prefix("https://github.com/").and_then(|s| s.strip_suffix("/issues")) else {
        return false;
    };
    let parts: Vec<_> = repo.split('/').collect();
    parts.len() == 2
        && parts.iter().all(|part| {
            !part.is_empty() && *part != "." && *part != ".." && part.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        })
        && !repo.eq_ignore_ascii_case("Aurora-Viewer/Aurora-Viewer")
        && api == format!("https://api.github.com/repos/{repo}/issues")
}

pub fn reporting_destination_configured() -> bool {
    reporting_destination(ISSUES_API, ISSUES_URL)
}

pub fn reporting_oauth_configured() -> bool {
    reporting_destination_configured() && !GITHUB_CLIENT_ID.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amante_security_reports_require_consistent_owner_controlled_destinations() {
        assert!(reporting_destination(
            "https://api.github.com/repos/example/amante-test/issues",
            "https://github.com/example/amante-test/issues"
        ));
        assert!(!reporting_destination(
            "https://api.github.com/repos/Aurora-Viewer/Aurora-Viewer/issues",
            "https://github.com/Aurora-Viewer/Aurora-Viewer/issues"
        ));
        assert!(!reporting_destination(
            "https://api.github.com/repos/other/repository/issues",
            "https://github.com/example/amante-test/issues"
        ));
        assert!(!reporting_destination("", ""));
    }

    #[test]
    fn amante_security_production_updates_cannot_be_activated_by_a_feed_url() {
        assert!(!std::hint::black_box(PRODUCTION_UPDATES_READY));
    }
}
