//! Defense in depth for PRIVATE local diagnostics, not a public-report sanitizer.
//! Public reports must keep using their closed vocabulary and explicit consent.
/// Only web/mail links may reach the OS URL handler. Files have separate,
/// explicit file-manager actions; SLURLs are dispatched inside the viewer.
pub fn external_web_url(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    !value.chars().any(char::is_control) && (lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:"))
}

pub fn redact_log(line: &str) -> std::borrow::Cow<'_, str> {
    let lower = line.to_ascii_lowercase();
    if [
        "password",
        "access_token",
        "refresh_token",
        "authorization",
        "cookie",
        "openid_token",
        "secure_session",
        "session_id",
        "sessionid",
        "mfa_token",
    ]
    .iter()
    .any(|word| lower.contains(word))
    {
        return "[sensitive diagnostic removed]\n".into();
    }
    let mut out = String::with_capacity(line.len());
    let mut offset = 0;
    while let Some((i, _)) = ["https://", "http://", "wss://", "ws://"]
        .iter()
        .filter_map(|s| lower[offset..].find(s).map(|i| (i, s)))
        .min_by_key(|(i, _)| *i)
    {
        let start = offset + i;
        out.push_str(&line[offset..start]);
        out.push_str("[URL removed]");
        let rest = &line[start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']'))
            .unwrap_or(rest.len());
        offset = start + end;
    }
    out.push_str(&line[offset..]);
    if out == line { line.into() } else { out.into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn amante_security_urls_and_credential_records_never_reach_log_sinks() {
        for input in [
            "network https://user:SYNTHETIC@example.invalid/private/capability?secret=VALUE timed out",
            "wss://example.invalid/SYNTHETIC disconnected",
            "request (https://example.invalid/SYNTHETIC): refused",
        ] {
            let redacted = redact_log(input);
            assert!(!redacted.contains("SYNTHETIC") && !redacted.contains("VALUE"));
        }
        for input in [
            "password=VALUE",
            "Authorization: Bearer VALUE",
            "Cookie: VALUE",
            "secure_session_id=VALUE",
        ] {
            assert!(!redact_log(input).contains("VALUE"));
        }
        assert_eq!(redact_log("plain diagnostic"), "plain diagnostic");
        assert_eq!(
            redact_log("échec https://example.invalid/secret timeout"),
            "échec [URL removed] timeout"
        );
    }

    #[test]
    fn amante_security_os_links_cannot_execute_files_or_custom_protocols() {
        for url in [
            "https://example.invalid/path",
            "HTTP://example.invalid",
            "mailto:bugs@example.invalid",
        ] {
            assert!(external_web_url(url));
        }
        for url in [
            "file:///tmp/run.exe",
            "javascript:alert(1)",
            "data:text/html,test",
            "ms-msdt:/test",
            "shell:Downloads",
            "/tmp/run",
            "https://example.invalid\nfile:///tmp/run",
        ] {
            assert!(!external_web_url(url));
        }
    }
}
