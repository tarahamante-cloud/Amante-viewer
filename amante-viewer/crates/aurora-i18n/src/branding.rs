//! Brand only bundled, application-owned copy. Never apply this to residents'
//! names, chat, inventory content, imported documents, or legal notices.

pub const PROJECT: &str = "https://github.com/tarahamante-cloud/Amante-viewer";

pub fn builtin(source: &str) -> String {
    source
        .replace("https://aurora-viewer.org", PROJECT)
        .replace(
            "https://github.com/Aurora-Viewer/Aurora-Viewer/wiki",
            &format!("{PROJECT}/blob/main/amante-viewer/docs/amante/README.md"),
        )
        .replace("https://github.com/Aurora-Viewer/Aurora-Viewer", PROJECT)
        .replace("Aurora", "Amante")
        // Keep Fluent IDs and compatibility environment switches unchanged.
        .replace("Usage: aurora-viewer", "Usage: amante-viewer")
        .replace("Utilisation : aurora-viewer", "Utilisation : amante-viewer")
        .replace("aurora-viewer [", "amante-viewer [")
        .replace("aurora-<text>.log", "amante-<text>.log")
        .replace("aurora-demo-<text>.log", "amante-demo-<text>.log")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_branding_preserves_message_ids_and_compatibility_switches() {
        let text = builtin("about-aurora-viewer = Aurora Viewer\ncli = Usage: aurora-viewer AURORA_DEMO=1");
        assert!(text.contains("about-aurora-viewer = Amante Viewer"));
        assert!(text.contains("Usage: amante-viewer AURORA_DEMO=1"));
        assert!(!text.contains("Aurora"));
    }

    #[test]
    fn bundled_links_never_rebrand_an_upstream_website_as_our_own() {
        let text = builtin("Aurora Viewer https://aurora-viewer.org https://github.com/Aurora-Viewer/Aurora-Viewer/wiki");
        assert!(text.contains(PROJECT));
        assert!(!text.contains("aurora-viewer.org"));
        assert!(!text.contains("Aurora-Viewer"));
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
            assert!(!branded.contains("Aurora"), "{code}");
            assert!(crate::text_in(code, "exit-title").contains("Amante"), "{code}");
        }
    }

    #[test]
    fn residents_names_are_not_rebranded_during_interpolation() {
        let mut args = crate::Args::new();
        args.set("name", "Aurora Resident");
        let rendered = crate::format_in("en", "demo-aurora-mention", &args);
        assert!(rendered.contains("Aurora Resident"));
    }
}
