//! Owner-controlled, public build configuration. No Aurora service fallback.
//! OAuth client IDs are public; tokens and signing keys do not belong here.

macro_rules! public_setting {
    ($name:ident, $env:literal, $default:literal) => {
        pub const $name: &str = match option_env!($env) {
            None => $default,
            Some(value) => {
                if value.is_empty() {
                    $default
                } else {
                    value
                }
            }
        };
    };
    ($name:ident, $env:literal) => {
        pub const $name: &str = match option_env!($env) {
            Some(value) => value,
            None => "",
        };
    };
}

// The owner authorized this public fork. Browser reporting can use its issue
// tracker without an OAuth application; direct sending still requires one.
public_setting!(
    ISSUES_URL,
    "AMANTE_ISSUES_URL",
    "https://github.com/tarahamante-cloud/Amante-viewer-native/issues"
);
public_setting!(
    ISSUES_API,
    "AMANTE_ISSUES_API",
    "https://api.github.com/repos/tarahamante-cloud/Amante-viewer-native/issues"
);
// Public Client ID supplied by the Amante owner, not the numeric App ID.
// Empty CI variables use this identity too; no secret belongs in the binary.
public_setting!(GITHUB_CLIENT_ID, "AMANTE_GITHUB_CLIENT_ID", "Iv23li2eYLSFqjYE8MDw");
public_setting!(RELEASES_API, "AMANTE_RELEASES_API");
public_setting!(RELEASE_PAGE, "AMANTE_RELEASE_PAGE");
public_setting!(RELEASE_DOWNLOADS, "AMANTE_RELEASE_DOWNLOADS");
// Separate from executable updates. The pack is pinned by size and SHA-256 in
// aurora-updater; publishing a viewer feed must not implicitly enable plugins.
public_setting!(MEDIA_RELEASE_DOWNLOADS, "AMANTE_MEDIA_RELEASE_DOWNLOADS");

/// Intentionally false until the owner pins usable PUBLIC keys in the updater's
/// compiled trust-root.json and reviews signing custody and release activation.
/// See docs/amante/AUTHENTICATED_UPDATES.md; authentication code alone is not
/// authorization to activate production releases.
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

/// A GitHub release-download prefix, not an arbitrary CDN or Aurora's repo.
/// This setting is public and must be supplied by the repository owner.
fn media_destination(downloads: &str) -> bool {
    downloads.strip_suffix("/releases/download").is_some_and(|repo| {
        reporting_destination(
            &format!(
                "{}/issues",
                repo.replacen("https://github.com/", "https://api.github.com/repos/", 1)
            ),
            &format!("{repo}/issues"),
        )
    })
}

pub fn media_distribution_configured() -> bool {
    media_destination(MEDIA_RELEASE_DOWNLOADS)
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
    fn amante_security_reporting_uses_owner_identity_with_public_build_override() {
        let expected = match option_env!("AMANTE_GITHUB_CLIENT_ID") {
            Some(value) if !value.is_empty() => value,
            _ => "Iv23li2eYLSFqjYE8MDw",
        };
        assert_eq!(GITHUB_CLIENT_ID, expected);
        assert_eq!(
            reporting_oauth_configured(),
            reporting_destination_configured() && !expected.trim().is_empty()
        );
    }

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

    #[test]
    fn amante_security_media_requires_an_independent_owner_destination() {
        assert!(media_destination("https://github.com/example/amante-media/releases/download"));
        for url in [
            "",
            "https://github.com/Aurora-Viewer/Aurora-Viewer/releases/download",
            "https://github.com/aurora-viewer/aurora-viewer/releases/download",
            "http://github.com/example/media/releases/download",
            "https://github.com/example/media/releases/download?token=example",
            "https://github.com/example/media/extra/releases/download",
            "https://github.com.evil.example/example/media/releases/download",
        ] {
            assert!(!media_destination(url), "{url}");
        }
    }
}
