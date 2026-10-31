//! Keep the digest-stack patch covered by upstream's independent test vectors.
//! These tests are offline: no microphone, grid, network or real credentials.

#[test]
fn amante_security_ring_provider_conforms_after_digest_upgrade() {
    rtc_crypto::conformance::assert_provider(&rtc_crypto::providers::RingProvider::new());
}

#[test]
fn amante_security_aws_lc_provider_conforms_after_digest_upgrade() {
    rtc_crypto::conformance::assert_provider(&rtc_crypto::providers::AwsLcRsProvider::new());
}
