use super::*;
use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair},
};

fn key() -> Ed25519KeyPair {
    // Test-only ephemeral keys. No private key material is written to disk.
    let bytes = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    Ed25519KeyPair::from_pkcs8(bytes.as_ref()).unwrap()
}
fn encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn root(key: &Ed25519KeyPair, n: u64) -> Root {
    let keys = BTreeMap::from([("key".into(), encode(key.public_key().as_ref()))]);
    Root {
        schema: 1,
        product: PRODUCT.into(),
        version: n,
        expires: 10000,
        root_threshold: 1,
        release_threshold: 1,
        root_keys: keys.clone(),
        release_keys: keys,
    }
}
fn envelope<T: Serialize>(value: &T, keys: &[&Ed25519KeyPair], domain: &[u8]) -> Envelope {
    let signed = serde_json::to_string(value).unwrap();
    let msg = [domain, signed.as_bytes()].concat();
    Envelope {
        signed,
        signatures: keys
            .iter()
            .map(|key| Signature {
                key: "key".into(),
                signature: encode(key.sign(&msg).as_ref()),
            })
            .collect(),
    }
}
fn snapshot(seq: u64, root_version: u64, ver: &str) -> Snapshot {
    Snapshot {
        schema: 1,
        product: PRODUCT.into(),
        root_version,
        sequence: seq,
        issued: 50,
        expires: 500,
        releases: vec![Target {
            version: ver.into(),
            platform: "windows-x64".into(),
            notes: "Signed notes".into(),
            url: "https://github.com/example/amante/releases".into(),
            archive: Asset {
                name: format!("Amante-Viewer-v{ver}-windows-x64.zip"),
                size: 123,
                browser_download_url: format!("https://github.com/example/amante/releases/download/v{ver}/package.zip"),
            },
            sha256: "ab".repeat(32),
            minimum_supported_version: Some("0.5.0".into()),
            cache_invalidation_before: Some("0.6.0".into()),
            settings_reset_before: None,
        }],
    }
}
fn bundle(s: &Snapshot, k: &Ed25519KeyPair) -> Vec<u8> {
    serde_json::to_vec(&Bundle {
        roots: vec![],
        releases: envelope(s, &[k], RELEASE_DOMAIN),
    })
    .unwrap()
}
fn check(anchor: Root, state: &mut State, bytes: &[u8], time: u64) -> Result<Option<Release>> {
    verify(&Source::default(), anchor, state, bytes, "0.1.0", Channel::Stable, time)
}

#[test]
fn signed_metadata_binds_all_target_fields_and_policy() {
    let k = key();
    let bytes = bundle(&snapshot(1, 1, "1.0.0"), &k);
    let r = check(root(&k, 1), &mut State::default(), &bytes, 100).unwrap().unwrap();
    assert!(r.required && r.maintenance.clear_cache && !r.maintenance.reset_settings);
    let auth = r.authentication.as_ref().unwrap();
    assert_eq!(auth.sha256, [0xab; 32]);
    assert_eq!(auth.digest, digest(&r).unwrap());
    let mut state = State {
        sequence: 1,
        last_time: 100,
        ..Default::default()
    };
    assert_eq!(auth.verify_state(&r, &state, 100).unwrap(), [0xab; 32]);
    assert!(auth.verify_state(&r, &state, 500).is_err());
    assert!(auth.verify_state(&r, &state, 99).is_err());
    state.sequence = 2;
    assert!(auth.verify_state(&r, &state, 100).is_err());
    state.sequence = 1;
    let mut changed = r.clone();
    changed.archive.as_mut().unwrap().size += 1;
    assert_ne!(auth.digest, digest(&changed).unwrap());
    assert!(auth.verify_state(&changed, &state, 100).is_err());
    changed = r.clone();
    changed.maintenance.reset_settings = true;
    assert_ne!(auth.digest, digest(&changed).unwrap());
    let mut tampered: Bundle = serde_json::from_slice(&bytes).unwrap();
    tampered.releases.signed = tampered.releases.signed.replace("Signed notes", "Forged notes");
    assert!(check(root(&k, 1), &mut State::default(), &serde_json::to_vec(&tampered).unwrap(), 100).is_err());
}

#[test]
fn rejects_unsigned_wrong_key_role_product_expired_future_and_oversized_metadata() {
    let k = key();
    let wrong = key();
    let s = snapshot(1, 1, "1.0.0");
    assert!(check(root(&k, 1), &mut State::default(), &bundle(&s, &wrong), 100).is_err());
    for domain in [ROOT_DOMAIN, b"".as_slice()] {
        let b = Bundle {
            roots: vec![],
            releases: envelope(&s, &[&k], domain),
        };
        assert!(check(root(&k, 1), &mut State::default(), &serde_json::to_vec(&b).unwrap(), 100).is_err());
    }
    let mut b = Bundle {
        roots: vec![],
        releases: envelope(&s, &[&k], RELEASE_DOMAIN),
    };
    b.releases.signatures.clear();
    assert!(check(root(&k, 1), &mut State::default(), &serde_json::to_vec(&b).unwrap(), 100).is_err());
    for time in [49, 500, 10001] {
        assert!(check(root(&k, 1), &mut State::default(), &bundle(&s, &k), time).is_err());
    }
    let mut s = s;
    s.product = "other-viewer".into();
    assert!(check(root(&k, 1), &mut State::default(), &bundle(&s, &k), 100).is_err());
    assert!(check(root(&k, 1), &mut State::default(), &vec![b' '; MAX_METADATA + 1], 100).is_err());
    assert!(pinned().is_err()); // The shipped disabled root must not become trust.
    assert!(Source::default().client().is_err());
}

#[test]
fn dual_signed_rotation_persists_and_revokes_old_release_keys() {
    let old = key();
    let new = key();
    let next = root(&new, 2);
    let rotation = envelope(&next, &[&old, &new], ROOT_DOMAIN);
    let b = Bundle {
        roots: vec![rotation.clone()],
        releases: envelope(&snapshot(2, 2, "2.0.0"), &[&new], RELEASE_DOMAIN),
    };
    let mut state = State::default();
    check(root(&old, 1), &mut state, &serde_json::to_vec(&b).unwrap(), 100).unwrap();
    assert_eq!(state.roots.len(), 1);
    let disk = serde_json::to_vec(&state).unwrap();
    let mut loaded: State = serde_json::from_slice(&disk).unwrap();
    let rotated_bundle = |s: Snapshot, k: &Ed25519KeyPair| {
        serde_json::to_vec(&Bundle {
            roots: vec![rotation.clone()],
            releases: envelope(&s, &[k], RELEASE_DOMAIN),
        })
        .unwrap()
    };
    check(root(&old, 1), &mut loaded, &rotated_bundle(snapshot(3, 2, "3.0.0"), &new), 101).unwrap();
    assert!(check(root(&old, 1), &mut loaded, &rotated_bundle(snapshot(4, 2, "4.0.0"), &old), 102).is_err());
    assert!(check(root(&old, 1), &mut loaded, &bundle(&snapshot(4, 1, "4.0.0"), &old), 102).is_err());
    for keys in [vec![&old], vec![&new]] {
        let b = Bundle {
            roots: vec![envelope(&next, &keys, ROOT_DOMAIN)],
            releases: envelope(&snapshot(2, 2, "2.0.0"), &[&new], RELEASE_DOMAIN),
        };
        assert!(check(root(&old, 1), &mut State::default(), &serde_json::to_vec(&b).unwrap(), 100).is_err());
    }
    let b = Bundle {
        roots: vec![envelope(&root(&new, 3), &[&old, &new], ROOT_DOMAIN)],
        releases: envelope(&snapshot(2, 3, "2.0.0"), &[&new], RELEASE_DOMAIN),
    };
    assert!(check(root(&old, 1), &mut State::default(), &serde_json::to_vec(&b).unwrap(), 100).is_err());
}

#[test]
fn thresholds_count_distinct_keys_not_duplicate_signatures() {
    let k = key();
    let mut r = root(&k, 1);
    let other = key();
    r.release_keys.insert("other".into(), encode(other.public_key().as_ref()));
    r.release_threshold = 2;
    let mut e = envelope(&snapshot(1, 1, "1.0.0"), &[&k, &k], RELEASE_DOMAIN);
    assert!(signatures(&e, &r.release_keys, 2, RELEASE_DOMAIN).is_err());
    let mut sig = envelope(&snapshot(1, 1, "1.0.0"), &[&other], RELEASE_DOMAIN).signatures.remove(0);
    sig.key = "other".into();
    e.signatures.push(sig);
    signatures(&e, &r.release_keys, 2, RELEASE_DOMAIN).unwrap();
    r.release_keys.insert("alias".into(), encode(k.public_key().as_ref()));
    assert!(root_valid(&r).is_err());
}

#[test]
fn rollback_equivocation_version_floor_and_clock_survive_restart() {
    let k = key();
    let mut state = State::default();
    let bytes = bundle(&snapshot(10, 1, "2.0.0"), &k);
    check(root(&k, 1), &mut state, &bytes, 100).unwrap();
    let path = std::env::temp_dir().join(format!("amante-auth-{}.json", uuid::Uuid::new_v4()));
    persist(&path, &state).unwrap();
    let mut state = read_state(&path).unwrap();
    fs::remove_file(path).unwrap();
    check(root(&k, 1), &mut state, &bytes, 101).unwrap(); // identical replay is safe until expiry
    for (seq, ver, time) in [(9, "3.0.0", 102), (10, "3.0.0", 102), (11, "1.0.0", 102), (11, "3.0.0", 99)] {
        assert!(check(root(&k, 1), &mut state, &bundle(&snapshot(seq, 1, ver), &k), time).is_err());
    }
    check(root(&k, 1), &mut state, &bundle(&snapshot(11, 1, "3.0.0"), &k), 102).unwrap();
}

#[test]
fn rejects_unsafe_assets_and_signed_invalid_policy() {
    let k = key();
    for url in ["http://github.com/a", "https://attacker.example/a", "file:///tmp/a"] {
        let mut s = snapshot(1, 1, "1.0.0");
        s.releases[0].archive.browser_download_url = url.into();
        assert!(check(root(&k, 1), &mut State::default(), &bundle(&s, &k), 100).is_err());
    }
    let mut s = snapshot(1, 1, "1.0.0");
    s.releases[0].settings_reset_before = Some("99.0.0".into());
    assert!(check(root(&k, 1), &mut State::default(), &bundle(&s, &k), 100).is_err());
}

#[test]
fn malformed_or_unwritable_state_never_silently_resets_trust() {
    let dir = std::env::temp_dir().join(format!("amante-state-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("state.json");
    fs::write(&path, b"not valid trust state").unwrap();
    assert!(read_state(&path).is_err());
    assert!(persist(&path.join("invalid-child"), &State::default()).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"not valid trust state");
    let lock = dir.join("state.lock");
    let _guard = OpenOptions::new().create_new(true).write(true).open(&lock).unwrap();
    assert!(OpenOptions::new().create_new(true).write(true).open(&lock).is_err());
    drop(_guard);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn signed_channels_keep_independent_floors_and_never_downgrade_installed_builds() {
    let k = key();
    let mut s = snapshot(1, 1, "1.0.0");
    s.releases.extend(snapshot(1, 1, "2.0.0-beta.1").releases);
    let bytes = bundle(&s, &k);
    let mut state = State::default();
    let verify_channel =
        |state: &mut State, channel, current| verify(&Source::default(), root(&k, 1), state, &bytes, current, channel, 100);
    let stable = verify_channel(&mut state, Channel::Stable, "0.1.0").unwrap().unwrap();
    assert_eq!(stable.version, "1.0.0");
    let beta = verify_channel(&mut state, Channel::Beta, "0.1.0").unwrap().unwrap();
    assert_eq!(beta.version, "2.0.0-beta.1");
    assert_eq!(state.stable.as_deref(), Some("1.0.0"));
    assert_eq!(state.beta.as_deref(), Some("2.0.0-beta.1"));
    assert!(verify_channel(&mut state, Channel::Stable, "1.0.0+local").unwrap().is_none());
    assert!(verify_channel(&mut state, Channel::Stable, "3.0.0").unwrap().is_none());
}
