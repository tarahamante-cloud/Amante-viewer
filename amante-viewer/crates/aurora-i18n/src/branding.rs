//! Brand only bundled, application-owned copy. Never apply this to residents'
//! names, chat, inventory content, imported documents, or legal notices.

/// Only these remaining product messages need a rendering-time substitution.
/// All other branding is explicit in the catalogs. In particular, NEVER replace
/// names in demo notecards, format names, credits or interpolated user content.
/// Inventory/add-on instructions below match the actual Amante-owned names
/// already used by this fork; this does not rename any existing user content.
const PRODUCT_MESSAGES: &[&str] = &[
    "ao-help",
    "ao-migrate-help",
    "ao-migrate-destination",
    "ao-migrate-name-exists",
    "ao-bootstrap-interrupted",
    "texture-library-empty",
    "texture-library-content-hint",
    "picker-aurora-tab",
    "picker-empty-library",
    "received-filing-help",
    "movelock-mode-bridge",
    "bridge-preparing",
    "bridge-repair",
    "bridge-reinstalling",
    "bridge-stopped",
    "radar-opt-bridge",
    "ui-options-aurora-build-help",
    "bridge-script-info-loading",
    "bridge-object-info-loading",
    "poser-bridge-needed",
    "poser-bridge-starting",
    "poser-bridge-silent",
    "poser-bridge-failed",
    "portal-bridge-required",
    "blender-shape-connect",
    "blender-live-bundled-version",
    "blender-live-instructions",
    "cli-help",
    "status-aurora-viewer-skin-theme-json-colors",
    "world-status-the-resident-you-messaged-has-enabled-aurora-viewer-s",
    "world-status-the-resident-you-messaged-has-enabled-aurora-viewer-s-2",
    "world-status-the-resident-you-messaged-has-enabled-aurora-viewer-s-3",
    "movelock-mode-sit",
    "movelock-sit-needs-stand",
];

/// Transform message values, never IDs, comments, attributes or interpolated
/// user content. The allowlist is intentionally small and independently tested.
pub fn builtin(source: &str) -> String {
    source
        .split_inclusive('\n')
        .map(|line| {
            let Some((key, value)) = line.split_once(" = ") else {
                return line.to_owned();
            };
            if !PRODUCT_MESSAGES.contains(&key) {
                return line.to_owned();
            }
            let mut value = value.replace("Aurora", "Amante");
            if key == "cli-help" {
                value = value
                    .replace("aurora-viewer [", "amante-viewer [")
                    .replace("aurora-demo-<", "amante-demo-<")
                    .replace("aurora-<", "amante-<");
            }
            format!("{key} = {value}")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_branding_preserves_message_ids_and_compatibility_switches() {
        let text = builtin("cli-help = Aurora Viewer Usage: aurora-viewer [--help] AURORA_DEMO=1\n");
        assert!(text.starts_with("cli-help = Amante Viewer"));
        assert!(text.contains("Usage: amante-viewer [--help] AURORA_DEMO=1\n"));
    }

    #[test]
    fn demo_content_and_technical_references_are_not_rebranded() {
        let text = "demo-card = Aurora Viewer https://aurora-viewer.org\n\
                    import-preview-aurora = Aurora 3D\n\
                    poser-file-filter = Aurora pose, native animation or BVH\n\
                    nameplate-aurora-staff = Aurora Viewer\n";
        assert_eq!(builtin(text), text);
    }

    #[test]
    fn every_locale_is_branded_without_changing_its_message_ids() {
        use fluent_syntax::ast::Entry;
        for (code, source) in crate::SOURCES {
            let original = fluent_syntax::parser::parse(*source).expect("original Fluent");
            let branded = builtin(source);
            let rewritten = fluent_syntax::parser::parse(branded.as_str()).expect("branded Fluent");
            let ids = |entries: &[Entry<&str>]| {
                entries
                    .iter()
                    .filter_map(|entry| match entry {
                        Entry::Message(message) => Some(message.id.name.to_owned()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(ids(&original.body), ids(&rewritten.body), "{code}");
            assert!(crate::text_in(code, "exit-title").contains("Amante"), "{code}");
            assert!(crate::text_in(code, "ui-bars-about").contains("Amante"), "{code}");
            assert!(crate::text_in(code, "ui-options-aurora-build-help").contains("Amante"), "{code}");
            assert!(crate::text_in(code, "crash-heading").contains("Amante"), "{code}");
            assert!(crate::text_in(code, "import-preview-aurora").contains("Aurora 3D"), "{code}");
            assert!(crate::text_in(code, "nameplate-aurora-staff").contains("Aurora"), "{code}");
            assert!(crate::text_in(code, "ao-help").contains("#Amante/Amante AO"), "{code}");
            let mut args = crate::Args::new();
            args.set("version", "1");
            assert!(
                crate::format_in(code, "blender-live-bundled-version", &args).contains("Amante LiveMesh"),
                "{code}"
            );
            assert!(crate::text_in(code, "bridge-repair").contains("Amante"), "{code}");
        }
    }

    #[test]
    fn residents_names_are_not_rebranded_during_interpolation() {
        let mut args = crate::Args::new();
        args.set("name", "Aurora Resident");
        let rendered = crate::format_in("en", "demo-aurora-mention", &args);
        assert!(rendered.contains("Aurora Resident"));
    }

    #[test]
    fn remaining_upstream_names_have_an_explicit_preservation_reason() {
        // These are content/compatibility identities, not the running product.
        let technical = ["import-preview-aurora", "poser-file-filter"];
        for (code, source) in crate::SOURCES {
            let branded = builtin(source);
            let mut key = "";
            for line in branded.lines() {
                if line.starts_with('#') {
                    continue;
                }
                let value = if !line.starts_with(char::is_whitespace) {
                    let Some((id, value)) = line.split_once(" = ") else { continue };
                    key = id;
                    value
                } else {
                    line
                };
                if !value.contains("Aurora") {
                    continue;
                }
                let preserved = technical.contains(&key)
                    || key.starts_with("demo-")
                    || key.starts_with("notecard-")
                    || key.starts_with("world-appearance-aurora-")
                    || key.starts_with("nameplate-aurora-staff")
                    || key.starts_with("photo-look")
                    || matches!(
                        key,
                        "app-aurora-elsewhere" | "common-aurora-demo" | "common-aurora-plaza" | "shape-demo-name" | "physics-demo-name"
                    );
                assert!(preserved, "{code}: unreviewed product name in {key}: {value}");
            }
        }
    }
}
