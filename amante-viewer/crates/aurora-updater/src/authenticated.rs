//! Exact-byte, domain-separated Ed25519 metadata. The empty compiled root is
//! deliberately unusable until the owner pins PUBLIC keys in a reviewed build.
use crate::feed::{Asset, Channel, Release, Source, threshold_applies, version};
use anyhow::{Context, Result, ensure};
use ring::signature::{ED25519, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const PRODUCT: &str = "amante-viewer";
const ROOT_DOMAIN: &[u8] = b"Amante update root v1\n";
const RELEASE_DOMAIN: &[u8] = b"Amante update releases v1\n";
const MAX_METADATA: usize = 1024 * 1024;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: u32,
    product: String,
    version: u64,
    expires: u64,
    root_threshold: usize,
    release_threshold: usize,
    root_keys: BTreeMap<String, String>,
    release_keys: BTreeMap<String, String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Signature {
    key: String,
    signature: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    /// UTF-8 JSON string. Sign these exact bytes, NOT re-serialized JSON.
    signed: String,
    signatures: Vec<Signature>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    roots: Vec<Envelope>,
    releases: Envelope,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: u32,
    product: String,
    root_version: u64,
    sequence: u64,
    issued: u64,
    expires: u64,
    releases: Vec<Target>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Target {
    version: String,
    platform: String,
    notes: String,
    url: String,
    archive: Asset,
    sha256: String,
    minimum_supported_version: Option<String>,
    cache_invalidation_before: Option<String>,
    settings_reset_before: Option<String>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct State {
    // Keep the signed rotation chain so disk cannot supply new unauthenticated keys.
    roots: Vec<Envelope>,
    sequence: u64,
    snapshot_hash: Vec<u8>,
    last_time: u64,
    stable: Option<String>,
    beta: Option<String>,
}

/// Only the verifier can construct this seal. A changed release loses authorization.
#[derive(Clone, Debug)]
pub struct Authorization {
    digest: Vec<u8>,
    sha256: [u8; 32],
    expires: u64,
    sequence: u64,
}

fn digest(release: &Release) -> Result<Vec<u8>> {
    Ok(Sha256::digest(serde_json::to_vec(release)?).to_vec())
}

pub(crate) fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

impl Authorization {
    pub(crate) fn verify(&self, release: &Release) -> Result<[u8; 32]> {
        let state = read_state(&state_path()?)?;
        self.verify_state(release, &state, now()?)
    }

    fn verify_state(&self, release: &Release, state: &State, time: u64) -> Result<[u8; 32]> {
        ensure!(
            time < self.expires && digest(release)? == self.digest,
            "expired or changed update authorization"
        );
        ensure!(
            state.sequence == self.sequence && time >= state.last_time,
            "update superseded or clock rolled back"
        );
        Ok(self.sha256)
    }
}

fn hex<const N: usize>(text: &str) -> Result<[u8; N]> {
    ensure!(
        text.len() == N * 2 && text.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid cryptographic encoding"
    );
    let mut out = [0; N];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)?;
    }
    Ok(out)
}

fn root_valid(root: &Root) -> Result<()> {
    ensure!(root.schema == 1 && root.product == PRODUCT && root.version > 0, "wrong update root");
    for (keys, threshold) in [(&root.root_keys, root.root_threshold), (&root.release_keys, root.release_threshold)] {
        ensure!(
            threshold > 0 && threshold <= keys.len() && keys.len() <= 16,
            "invalid trust threshold"
        );
        let mut unique = BTreeSet::new();
        for (id, key) in keys {
            ensure!(!id.is_empty() && unique.insert(hex::<32>(key)?), "duplicate trust key");
        }
    }
    Ok(())
}

fn signatures(env: &Envelope, keys: &BTreeMap<String, String>, threshold: usize, domain: &[u8]) -> Result<()> {
    ensure!(
        env.signed.len() <= MAX_METADATA && env.signatures.len() <= 32,
        "oversized signatures"
    );
    let message = [domain, env.signed.as_bytes()].concat();
    let mut accepted = BTreeSet::new();
    for sig in &env.signatures {
        if let Some(key) = keys.get(&sig.key)
            && UnparsedPublicKey::new(&ED25519, hex::<32>(key)?)
                .verify(&message, &hex::<64>(&sig.signature)?)
                .is_ok()
        {
            accepted.insert(sig.key.as_str());
        }
    }
    ensure!(threshold > 0 && accepted.len() >= threshold, "update signature threshold not met");
    Ok(())
}

fn rotate(mut root: Root, chain: &[Envelope]) -> Result<Root> {
    ensure!(chain.len() <= 64, "too many root rotations");
    root_valid(&root)?;
    for env in chain {
        signatures(env, &root.root_keys, root.root_threshold, ROOT_DOMAIN)?;
        let next: Root = serde_json::from_str(&env.signed)?;
        root_valid(&next)?;
        ensure!(Some(next.version) == root.version.checked_add(1), "nonconsecutive root rotation");
        signatures(env, &next.root_keys, next.root_threshold, ROOT_DOMAIN)?;
        root = next;
    }
    Ok(root)
}

fn pinned() -> Result<Root> {
    let root: Root = serde_json::from_str(include_str!("trust-root.json"))?;
    root_valid(&root)?;
    Ok(root)
}

/// Activation requires code review of both the explicit flag and compiled root.
/// No environment-variable URL, remote response or user setting supplies trust.
pub(crate) fn activation(source: &Source) -> Result<()> {
    ensure!(
        aurora_version::amante_config::PRODUCTION_UPDATES_READY,
        "Amante production updates are inactive"
    );
    pinned()?;
    source.validate_url(crate::feed::RELEASES)?;
    source.validate_url(crate::feed::RELEASE_PAGE)?;
    source.validate_url(aurora_version::amante_config::RELEASE_DOWNLOADS)?;
    for url in [
        crate::feed::RELEASES,
        crate::feed::RELEASE_PAGE,
        aurora_version::amante_config::RELEASE_DOWNLOADS,
    ] {
        let url = reqwest::Url::parse(url)?;
        ensure!(url.query().is_none() && url.fragment().is_none(), "invalid update endpoint");
    }
    let repo = crate::feed::RELEASE_PAGE
        .strip_prefix("https://github.com/")
        .and_then(|s| s.strip_suffix("/releases"))
        .context("invalid Amante release repository")?;
    let parts: Vec<_> = repo.split('/').collect();
    ensure!(
        parts.len() == 2
            && parts
                .iter()
                .all(|s| !s.is_empty() && *s != "." && *s != ".." && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c)))
            && !repo.eq_ignore_ascii_case("Aurora-Viewer/Aurora-Viewer"),
        "invalid Amante release repository"
    );
    ensure!(
        aurora_version::amante_config::RELEASE_DOWNLOADS == format!("{}/download", crate::feed::RELEASE_PAGE)
            && crate::feed::RELEASES == format!("{}/latest/download/amante-updates.json", crate::feed::RELEASE_PAGE),
        "inconsistent Amante update endpoints"
    );
    Ok(())
}

fn state_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("org", "Amante", "AmanteViewer").context("update state directory unavailable")?;
    Ok(dirs.data_local_dir().join("update-trust").join("state.json"))
}

fn read_state(path: &Path) -> Result<State> {
    match fs::read(path) {
        Ok(bytes) => {
            ensure!(bytes.len() <= 4 * MAX_METADATA, "oversized update state");
            Ok(serde_json::from_slice(&bytes)?)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
        Err(e) => Err(e.into()),
    }
}

struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn persist(path: &Path, state: &State) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.write_all(&serde_json::to_vec(state)?)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        fs::File::open(path.parent().context("state parent")?)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

pub(crate) async fn check(source: &Source, current: &str, channel: Channel) -> Result<Option<Release>> {
    let client = source.client()?;
    let bytes = source.bytes(&client, crate::feed::RELEASES, MAX_METADATA).await?;
    let path = state_path()?;
    fs::create_dir_all(path.parent().context("state parent")?)?;
    let lock_path = path.with_extension("lock");
    let _file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .context("update trust state busy; do not delete its state")?;
    let _guard = Lock(lock_path);
    let mut state = read_state(&path)?;
    let release = verify(source, pinned()?, &mut state, &bytes, current, channel, now()?)?;
    // Fail closed: no release is offered if durable rollback protection failed.
    persist(&path, &state)?;
    Ok(release)
}

#[cfg(test)]
mod tests;

fn verify(
    source: &Source,
    anchor: Root,
    state: &mut State,
    bytes: &[u8],
    current: &str,
    channel: Channel,
    time: u64,
) -> Result<Option<Release>> {
    ensure!(
        bytes.len() <= MAX_METADATA && time >= state.last_time,
        "oversized metadata or clock rollback"
    );
    let installed = version(current).context("invalid installed version")?;
    let bundle: Bundle = serde_json::from_slice(bytes)?;
    // Every publication carries the complete chain from the compiled anchor,
    // so the same feed works for both first-time and returning installations.
    rotate(anchor.clone(), &state.roots)?;
    ensure!(
        bundle.roots.len() >= state.roots.len() && state.roots.iter().zip(&bundle.roots).all(|(old, next)| old.signed == next.signed),
        "root chain rollback or equivocation"
    );
    let root = rotate(anchor, &bundle.roots)?;
    ensure!(time < root.expires, "expired trust root");
    signatures(&bundle.releases, &root.release_keys, root.release_threshold, RELEASE_DOMAIN)?;
    let snap: Snapshot = serde_json::from_str(&bundle.releases.signed)?;
    ensure!(
        snap.schema == 1 && snap.product == PRODUCT && snap.root_version == root.version,
        "wrong release authority"
    );
    ensure!(
        snap.issued <= time && time < snap.expires && snap.expires.saturating_sub(snap.issued) <= 7 * 86400,
        "invalid release validity"
    );
    let hash = Sha256::digest(bundle.releases.signed.as_bytes()).to_vec();
    ensure!(snap.sequence > 0 && snap.sequence >= state.sequence, "release sequence rollback");
    ensure!(
        snap.sequence != state.sequence || state.snapshot_hash == hash,
        "release sequence equivocation"
    );
    ensure!(snap.releases.len() <= 100, "too many targets");
    let mut selected = None;
    let mut seen = BTreeSet::new();
    for target in snap.releases {
        let v = version(&target.version).context("invalid target version")?;
        ensure!(target.version == v.to_string() && v.build.is_empty(), "noncanonical target version");
        ensure!(seen.insert((target.platform.clone(), target.version.clone())), "duplicate target");
        source.validate_url(&target.url)?;
        source.validate_url(&target.archive.browser_download_url)?;
        let sha256 = crate::package::parse_sha256(&target.sha256)?;
        ensure!(
            target.archive.name == format!("Amante-Viewer-v{v}-windows-x64.zip")
                && target.archive.size > 0
                && target.archive.size <= crate::feed::MAX_ARCHIVE,
            "invalid signed archive"
        );
        if target.platform != "windows-x64" || !channel.accepts(&v) {
            continue;
        }
        if selected
            .as_ref()
            .is_none_or(|(best, _, _): &(semver::Version, Target, [u8; 32])| v.cmp_precedence(best).is_gt())
        {
            selected = Some((v, target, sha256));
        }
    }
    let floor = match channel {
        Channel::Stable => &mut state.stable,
        Channel::Beta => &mut state.beta,
    };
    let mut release = None;
    if let Some((v, target, sha256)) = selected {
        if let Some(previous) = floor.as_deref() {
            ensure!(
                !v.cmp_precedence(&version(previous).context("corrupt version floor")?).is_lt(),
                "target version rollback"
            );
        }
        let maintenance = crate::Maintenance {
            clear_cache: threshold_applies(target.cache_invalidation_before.as_deref(), current, &v)?,
            reset_settings: threshold_applies(target.settings_reset_before.as_deref(), current, &v)?,
        };
        let required = if let Some(min) = target.minimum_supported_version.as_deref() {
            let min = version(min).context("invalid minimum version")?;
            ensure!(
                min.pre.is_empty() && min.build.is_empty() && !min.cmp_precedence(&v).is_gt(),
                "invalid update floor"
            );
            installed.cmp_precedence(&min).is_lt()
        } else {
            false
        };
        *floor = Some(v.to_string());
        if v.cmp_precedence(&installed).is_gt() {
            let mut r = Release {
                tag: format!("v{v}"),
                version: v.to_string(),
                notes: target.notes,
                url: target.url,
                archive: Some(target.archive),
                checksum: None,
                required,
                maintenance,
                authentication: None,
            };
            r.authentication = Some(Authorization {
                digest: digest(&r)?,
                sha256,
                expires: snap.expires.min(root.expires),
                sequence: snap.sequence,
            });
            release = Some(r);
        }
    } else {
        ensure!(floor.is_none(), "previously published channel disappeared");
    }
    state.roots = bundle.roots;
    state.sequence = snap.sequence;
    state.snapshot_hash = hash;
    state.last_time = time;
    Ok(release)
}
