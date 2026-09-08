//! Regression contract for libass-compatible color argument boundaries.

use ass_core::analysis::events::tags::{parse_override_block, OverrideTag, TagDiagnostic};

fn check(
    parse: impl for<'a> Fn(&'a str, usize, &mut Vec<OverrideTag<'a>>, &mut Vec<TagDiagnostic<'a>>),
) {
    for name in ["c", "1c", "2c", "3c", "4c", "alpha", "1a", "2a", "3a", "4a"] {
        for arg in ["FF", "ff", "A0", "0A", "FFFFFFF", "&HAB&", "H", "lhpa", ""] {
            let input = format!("\\{name}{arg}\\fnArial");
            let mut tags = Vec::new();
            let mut diagnostics = Vec::new();
            parse(&input, 7, &mut tags, &mut diagnostics);
            assert!(diagnostics.is_empty(), "{input}");
            assert_eq!(tags.len(), 2, "{input}");
            assert_eq!(
                (tags[0].name(), tags[0].args(), tags[0].position()),
                (name, arg, 7),
                "{input}"
            );
            assert_eq!((tags[1].name(), tags[1].args()), ("fn", "Arial"));
        }
    }
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();
    parse(
        r"\clip(1,2,3,4)\iclip(1,2,3,4)\frz45\fscx120\unknown42",
        0,
        &mut tags,
        &mut diagnostics,
    );
    assert_eq!(
        tags.iter()
            .map(|t| (t.name(), t.args()))
            .collect::<Vec<_>>(),
        vec![
            ("clip", "(1,2,3,4)"),
            ("iclip", "(1,2,3,4)"),
            ("frz", "45"),
            ("fscx", "120"),
            ("unknown", "42")
        ]
    );
}

#[test]
fn standard_color_names_stop_before_hexadecimal_arguments() {
    check(parse_override_block);
}

#[cfg(feature = "plugins")]
#[test]
fn registry_color_names_stop_before_hexadecimal_arguments() {
    check(|input, offset, tags, diagnostics| {
        ass_core::analysis::events::tags::parse_override_block_with_registry(
            input,
            offset,
            tags,
            diagnostics,
            None,
        );
    });
}

#[cfg(feature = "plugins")]
mod extensions {
    use ass_core::{
        analysis::events::tags::parse_override_block_with_registry,
        plugin::{ExtensionRegistry, TagHandler, TagResult},
    };

    struct Custom(&'static str, TagResult);

    impl TagHandler for Custom {
        fn name(&self) -> &'static str {
            self.0
        }

        fn process(&self, args: &str) -> TagResult {
            assert_eq!(args, "42");
            self.1.clone()
        }
    }

    #[test]
    fn registered_extensions_are_not_reinterpreted_as_builtin_color_prefixes() {
        for result in [TagResult::Processed, TagResult::Ignored] {
            for name in ["custom", "alphaCustom", "1aCustom"] {
                let mut registry = ExtensionRegistry::new();
                registry
                    .register_tag_handler(Box::new(Custom(name, result.clone())))
                    .unwrap();
                let input = format!("\\{name}42\\alphaFF");
                let mut tags = Vec::new();
                let mut diagnostics = Vec::new();
                parse_override_block_with_registry(
                    &input,
                    0,
                    &mut tags,
                    &mut diagnostics,
                    Some(&registry),
                );
                assert!(diagnostics.is_empty());
                assert_eq!(tags.len(), 2);
                assert_eq!((tags[0].name(), tags[0].args()), (name, "42"));
                assert_eq!((tags[1].name(), tags[1].args()), ("alpha", "FF"));
            }
        }
    }

    #[test]
    fn failing_extension_keeps_diagnostic_and_does_not_hide_following_color() {
        let mut registry = ExtensionRegistry::new();
        registry
            .register_tag_handler(Box::new(Custom(
                "custom",
                TagResult::Failed("invalid".into()),
            )))
            .unwrap();
        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block_with_registry(
            r"\custom42\alphaFF",
            5,
            &mut tags,
            &mut diagnostics,
            Some(&registry),
        );
        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].name(), tags[0].args()), ("custom", "42"));
        assert_eq!((tags[1].name(), tags[1].args()), ("alpha", "FF"));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].offset, 5);
        assert_eq!(diagnostics[0].span, r"\custom42");
    }
}
