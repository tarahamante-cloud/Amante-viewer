# RTC crypto dependency patch

Source: the published `rtc-crypto` 0.21.1 crate, corresponding to upstream
https://github.com/webrtc-rs/rtc/tree/2a732df55ca4bcf26b80914fa0f570cfaf74cfcb/rtc-crypto

Retained under its upstream MIT OR Apache-2.0 licence. This copy preserves the
upstream crypto algorithms, provider interface, implementation and test vectors.
Only HMAC, MD5 and SHA1 are upgraded to the maintained digest stack, with the
HMAC constructor's `KeyInit` import adjusted for that version. This removes
`block-buffer` versions affected by GHSA-qwgh-2vcv-g2f7.

Remove the workspace patch once an upstream release includes these dependency
upgrades. Run both the provider conformance suite and Amante's voice tests when
updating it. MD5/SHA1 remain solely for existing WebRTC protocol compatibility;
no new protocol or application use of these algorithms is introduced here.
