//! A bounded GitHub release list, filtered by SemVer rather than GitHub's
//! creation date (the assets-only release is not a viewer): stable versions,
//! plus the `-beta.…` pre-releases for the Beta channel.

use anyhow::{Context, Result, ensure};
use reqwest::{Client, Url};
use semver::Version;
use serde::Deserialize;
use std::time::Duration;

pub const RELEASES: &str = aurora_version::amante_config::RELEASES_API;
pub const RELEASE_PAGE: &str = aurora_version::amante_config::RELEASE_PAGE;
const DOWNLOADS: &str = aurora_version::amante_config::RELEASE_DOWNLOADS;
/// Notification-only discovery is independent of executable replacement.
pub const NOTIFICATION_PAGE: &str = "https://github.com/tarahamante-cloud/Amante-viewer/releases";
const NOTIFICATION_API: &str = "https://api.github.com/repos/tarahamante-cloud/Amante-viewer/releases?per_page=100";
pub const POLICY: &str = "aurora-update-policy.json";
pub const MAX_ARCHIVE: u64 = 2 * 1024 * 1024 * 1024;

/// Which releases a viewer follows. Beta also takes the stable versions: a
/// stable release ranks above the betas of its own number (SemVer), so a
/// beta tester moves on to it once it is out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Channel {
    #[default]
    Stable,
    Beta,
}

/// The pre-release label of the Beta channel (`v0.7.7-beta.20261025.1`,
/// built by the Nightly workflow).
pub const BETA: &str = "beta";

impl Channel {
    /// Whether a release version belongs to this channel. Build metadata
    /// never does: it marks a development build, not a release.
    pub fn accepts(self, v: &Version) -> bool {
        v.build.is_empty() && (v.pre.is_empty() || (self == Channel::Beta && is_beta(v)))
    }
}

/// A Beta channel pre-release: `-beta`, `-beta.<…>`.
pub fn is_beta(v: &Version) -> bool {
    v.pre.as_str().split('.').next() == Some(BETA)
}

#[derive(Clone)]
pub struct Source {
    url: String,
    loopback: bool,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            url: RELEASES.into(),
            loopback: false,
        }
    }
}

impl Source {
    /// Test feeds are explicit loopback HTTP only; no arbitrary remote override.
    pub fn local(url: &str) -> Result<Self> {
        let url = Url::parse(url)?;
        ensure!(
            url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost")),
            "test feed must use loopback HTTP"
        );
        ensure!(url.username().is_empty() && url.password().is_none(), "credentials in feed URL");
        Ok(Self {
            url: url.into(),
            loopback: true,
        })
    }

    pub fn validate_url(&self, value: &str) -> Result<()> {
        let url = Url::parse(value)?;
        ensure!(url.username().is_empty() && url.password().is_none(), "credentials in update URL");
        if self.loopback {
            let base = Url::parse(&self.url)?;
            ensure!(url.origin() == base.origin(), "test asset outside loopback feed");
        } else {
            ensure!(
                url.scheme() == "https"
                    && matches!(
                        url.host_str(),
                        Some("api.github.com" | "github.com" | "release-assets.githubusercontent.com" | "objects.githubusercontent.com")
                    ),
                "untrusted update URL"
            );
        }
        Ok(())
    }

    /// Address of a file of the release `tag`, known without asking the feed
    /// (a test feed serves it from its own origin).
    pub fn asset_url(&self, tag: &str, name: &str) -> Result<String> {
        let path = format!("{tag}/{name}");
        if self.loopback {
            Ok(Url::parse(&self.url)?.join(&format!("/download/{path}"))?.into())
        } else {
            Ok(format!("{DOWNLOADS}/{path}"))
        }
    }

    pub fn client(&self) -> Result<Client> {
        if !self.loopback {
            crate::authenticated::activation(self)?;
        }
        self.http_client()
    }

    pub(crate) fn is_local(&self) -> bool {
        self.loopback
    }

    /// Recheck immediately before handing an already-prepared package to the
    /// installer. A long-open Restart offer must not outlive its authority.
    pub fn authorize_install(&self, release: &Release) -> Result<()> {
        if !self.loopback {
            crate::authenticated::activation(self)?;
            release.authentication.as_ref().context("unsigned update offer")?.verify(release)?;
        }
        Ok(())
    }

    /// Only media_pack uses this path. It never enables release discovery or
    /// executable replacement, which still require `client()`'s update guard.
    pub(crate) fn media_client(&self) -> Result<Client> {
        ensure!(
            self.loopback || aurora_version::amante_config::media_distribution_configured(),
            "Amante media distribution is pending owner configuration"
        );
        self.http_client()
    }

    pub(crate) fn media_asset_url(&self, tag: &str, name: &str) -> Result<String> {
        if self.loopback {
            self.asset_url(tag, name)
        } else {
            ensure!(
                aurora_version::amante_config::media_distribution_configured(),
                "Amante media distribution is pending owner configuration"
            );
            Ok(format!("{}/{tag}/{name}", aurora_version::amante_config::MEDIA_RELEASE_DOWNLOADS))
        }
    }

    fn http_client(&self) -> Result<Client> {
        let source = self.clone();
        let builder = Client::builder()
            .user_agent(aurora_version::user_agent())
            .https_only(!self.loopback)
            .connect_timeout(Duration::from_secs(8))
            .read_timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= 5 {
                    attempt.error("too many redirects")
                } else if source.validate_url(attempt.url().as_str()).is_ok() {
                    attempt.follow()
                } else {
                    attempt.error("untrusted redirect")
                }
            }));
        // An explicit local feed must not leave loopback through a system proxy.
        let builder = if self.loopback { builder.no_proxy() } else { builder };
        Ok(builder.build()?)
    }

    pub async fn bytes(&self, client: &Client, url: &str, limit: usize) -> Result<Vec<u8>> {
        self.validate_url(url)?;
        let mut response = client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?
            .error_for_status()?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            ensure!(bytes.len().saturating_add(chunk.len()) <= limit, "update metadata too large");
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }

    pub async fn check(&self, current: &str, channel: Channel) -> Result<Option<Release>> {
        if !self.loopback {
            return crate::authenticated::check(self, current, channel).await;
        }
        let client = self.client()?;
        let releases: Vec<GithubRelease> = serde_json::from_slice(&self.bytes(&client, &self.url, 4 * 1024 * 1024).await?)?;
        let Some((v, r)) = releases
            .into_iter()
            .filter_map(|r| {
                let v = version(&r.tag_name)?;
                // GitHub's pre-release flag must agree with the version: a
                // stable number marked pre-release (or the reverse) is skipped.
                (r.tag_name.starts_with('v') && !r.draft && r.prerelease == !v.pre.is_empty() && channel.accepts(&v)).then_some((v, r))
            })
            .max_by(|a, b| a.0.cmp_precedence(&b.0))
        else {
            return Ok(None);
        };
        if !super::is_newer(&v.to_string(), current) {
            return Ok(None);
        }
        self.validate_url(&r.html_url)?;
        let expected = format!("Amante-Viewer-{}-windows-x64.zip", r.tag_name);
        let archive = r
            .assets
            .iter()
            .find(|a| a.name == expected && a.size > 0 && a.size <= MAX_ARCHIVE)
            .cloned();
        let checksum = r.assets.iter().find(|a| a.name == format!("{expected}.sha256")).cloned();
        let mut required = false;
        let mut maintenance = super::Maintenance::default();
        if let Some(asset) = r.assets.iter().find(|a| a.name == POLICY) {
            let policy: Policy = serde_json::from_slice(&self.bytes(&client, &asset.browser_download_url, 16 * 1024).await?)?;
            ensure!(policy.schema == 1 && policy.version == v.to_string(), "incompatible update policy");
            maintenance.clear_cache = threshold_applies(policy.cache_invalidation_before.as_deref(), current, &v)?;
            maintenance.reset_settings = threshold_applies(policy.settings_reset_before.as_deref(), current, &v)?;
            if let Some(min) = policy.minimum_supported_version {
                let min = version(&min).context("invalid minimum supported version")?;
                ensure!(
                    min.pre.is_empty() && min.build.is_empty() && !min.cmp_precedence(&v).is_gt(),
                    "invalid update floor"
                );
                required = version(current).is_some_and(|c| c.cmp_precedence(&min).is_lt());
            }
        }
        for a in [&archive, &checksum].into_iter().flatten() {
            self.validate_url(&a.browser_download_url)?;
        }
        Ok(Some(Release {
            tag: r.tag_name,
            version: v.to_string(),
            notes: r.body.unwrap_or_default(),
            url: r.html_url,
            archive,
            checksum,
            required,
            maintenance,
            authentication: None,
        }))
    }
}

/// Read public release metadata only. Never fetch policies, packages or helpers.
/// Links are constructed from the fixed Amante repository, not feed-provided URLs.
pub async fn check_notification(current: &str, channel: Channel) -> Result<Option<Release>> {
    ensure!(version(current).is_some(), "invalid installed version");
    let source = Source {
        url: NOTIFICATION_API.into(),
        loopback: false,
    };
    let client = source.http_client()?;
    let releases = serde_json::from_slice(&source.bytes(&client, NOTIFICATION_API, 4 * 1024 * 1024).await?)?;
    Ok(notification_release(releases, current, channel))
}

fn notification_release(releases: Vec<GithubRelease>, current: &str, channel: Channel) -> Option<Release> {
    let (v, r) = releases
        .into_iter()
        .filter_map(|r| {
            let v = version(&r.tag_name)?;
            let expected = format!("Amante-Viewer-{}-windows-x64.zip", r.tag_name);
            (r.tag_name.starts_with('v')
                && !r.draft
                && r.prerelease == !v.pre.is_empty()
                && channel.accepts(&v)
                && r.assets.iter().any(|a| a.name == expected && a.size > 0 && a.size <= MAX_ARCHIVE))
            .then_some((v, r))
        })
        .max_by(|a, b| a.0.cmp_precedence(&b.0))?;
    if !super::is_newer(&v.to_string(), current) {
        return None;
    }
    Some(Release {
        url: format!("{NOTIFICATION_PAGE}/tag/{}", r.tag_name),
        tag: r.tag_name,
        version: v.to_string(),
        notes: r.body.unwrap_or_default(),
        archive: None,
        checksum: None,
        required: false,
        maintenance: Default::default(),
        authentication: None,
    })
}

pub fn version(value: &str) -> Option<Version> {
    Version::parse(value.strip_prefix('v').unwrap_or(value)).ok()
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Policy {
    schema: u32,
    version: String,
    minimum_supported_version: Option<String>,
    cache_invalidation_before: Option<String>,
    settings_reset_before: Option<String>,
}

pub(crate) fn threshold_applies(threshold: Option<&str>, current: &str, latest: &Version) -> Result<bool> {
    let Some(threshold) = threshold else { return Ok(false) };
    let cutoff = version(threshold).context("invalid maintenance threshold")?;
    ensure!(
        cutoff.pre.is_empty() && cutoff.build.is_empty() && !cutoff.cmp_precedence(latest).is_gt(),
        "invalid maintenance threshold"
    );
    Ok(version(current).is_some_and(|installed| installed.cmp_precedence(&cutoff).is_lt()))
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Release {
    pub tag: String,
    pub version: String,
    pub notes: String,
    pub url: String,
    pub archive: Option<Asset>,
    pub checksum: Option<Asset>,
    pub required: bool,
    pub maintenance: super::Maintenance,
    #[serde(skip)]
    pub authentication: Option<crate::authenticated::Authorization>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn notice(tag: &str, draft: bool, prerelease: bool, package: bool) -> GithubRelease {
        serde_json::from_value(serde_json::json!({
            "tag_name": tag,
            "draft": draft,
            "prerelease": prerelease,
            "html_url": "https://untrusted.example/ignore-this",
            "body": "Release notes",
            "assets": if package { vec![serde_json::json!({
                "name": format!("Amante-Viewer-{tag}-windows-x64.zip"),
                "size": 1234,
                "browser_download_url": "https://untrusted.example/never-fetch"
            })] } else { vec![] }
        }))
        .expect("fixture")
    }

    #[test]
    fn notification_supports_first_development_download_and_never_installs() {
        let r =
            notification_release(vec![notice("v0.1.0", false, false, true)], "0.0.0+6.g755da7b22", Channel::Stable).expect("new release");
        assert_eq!(r.url, format!("{NOTIFICATION_PAGE}/tag/v0.1.0"));
        assert_eq!(r.notes, "Release notes");
        assert!(r.archive.is_none() && r.checksum.is_none());
        assert!(!r.required && !r.maintenance.clear_cache && !r.maintenance.reset_settings);
        assert!(!std::hint::black_box(aurora_version::amante_config::PRODUCTION_UPDATES_READY));
        assert!(Source::default().client().is_err());
    }

    #[test]
    fn notification_suppresses_same_older_empty_and_unpublished_versions() {
        for current in ["0.2.0", "0.2.0+3.gabcdef", "0.3.0"] {
            assert!(notification_release(vec![notice("v0.2.0", false, false, true)], current, Channel::Stable).is_none());
        }
        for release in [
            notice("v1.0.0", true, false, true),
            notice("v1.0.0", false, false, false),
            notice("v1.0.0", false, true, true),
            notice("v1.0.0+2.gabc", false, false, true),
            notice("v9.0.0/../../evil", false, false, true),
        ] {
            assert!(notification_release(vec![release], "0.0.0", Channel::Stable).is_none());
        }
        assert!(notification_release(vec![], "0.0.0", Channel::Stable).is_none());
    }

    #[test]
    fn notification_selects_newest_ready_release_in_selected_channel() {
        let releases = || {
            vec![
                notice("v0.3.0", false, false, true),
                notice("v0.2.0", false, false, true),
                notice("v0.4.0-beta.1", false, true, true),
                notice("v9.0.0", false, false, false),
            ]
        };
        assert_eq!(
            notification_release(releases(), "0.1.0", Channel::Stable).expect("stable").version,
            "0.3.0"
        );
        assert_eq!(
            notification_release(releases(), "0.1.0", Channel::Beta).expect("beta").version,
            "0.4.0-beta.1"
        );
    }

    #[test]
    fn maintenance_thresholds_survive_skipped_releases_and_do_not_repeat() {
        let latest = Version::parse("0.5.0").expect("fixture");
        assert!(threshold_applies(Some("0.3.0"), "0.2.0+8.gabcdef", &latest).expect("policy"));
        assert!(!threshold_applies(Some("0.3.0"), "0.3.0", &latest).expect("policy"));
        assert!(!threshold_applies(None, "0.2.0", &latest).expect("old policy"));
        for invalid in ["0.6.0", "garbage", "0.3.0-rc.1", "0.3.0+build"] {
            assert!(threshold_applies(Some(invalid), "0.2.0", &latest).is_err());
        }
    }
    #[test]
    fn strict_semver_and_provenance() {
        assert!(crate::is_newer("v0.3.0", "0.2.0+8.gabc123"));
        assert!(!crate::is_newer("v0.2.0", "0.2.0+8.gabc123"));
        assert!(crate::is_newer("v1.0.0", "1.0.0-rc.1"));
        for tag in ["assets-1", "1.0", "v01.2.3", "1.2.3garbage"] {
            assert!(!crate::is_newer(tag, "0.1.0"));
        }
    }
    #[test]
    fn channels_take_stable_versions_and_beta_only_its_own_prereleases() {
        let v = |s: &str| version(s).expect("fixture");
        for (tag, stable, beta) in [
            ("v0.7.6", true, true),
            ("v0.7.7-beta.20261025.1", false, true),
            ("v0.7.7-beta", false, true),
            ("v0.7.7-rc.1", false, false),
            ("v0.7.7-betafoo.1", false, false),
            ("v0.7.6+3.gabcdef", false, false),
            ("v0.7.7-beta.1+3.gabcdef", false, false),
        ] {
            assert_eq!(Channel::Stable.accepts(&v(tag)), stable, "{tag}");
            assert_eq!(Channel::Beta.accepts(&v(tag)), beta, "{tag}");
        }
        // A stable release ranks above the betas of its number.
        assert!(crate::is_newer("v0.7.7", "0.7.7-beta.20261025.9"));
        assert!(crate::is_newer("v0.7.7-beta.20261026.1", "0.7.7-beta.20261025.9"));
        assert!(crate::is_newer("v0.7.7-beta.20261025.10", "0.7.7-beta.20261025.9"));
    }

    #[test]
    fn reject_external_or_credentialled_assets() {
        let production = Source::default();
        assert!(
            production
                .validate_url("https://github.com/Aurora-Viewer/Aurora-Viewer/releases")
                .is_ok()
        );
        for url in [
            "http://github.com/a",
            "https://example.org/a",
            "https://name:password@github.com/a",
            "file:///a",
        ] {
            assert!(production.validate_url(url).is_err());
        }
        let local = Source::local("http://127.0.0.1:8099/releases").expect("loopback");
        assert!(local.validate_url("http://127.0.0.1:8099/package.zip").is_ok());
        assert!(local.validate_url("http://127.0.0.1:8100/package.zip").is_err());
        assert!(Source::local("https://example.org/feed").is_err());
    }
}
