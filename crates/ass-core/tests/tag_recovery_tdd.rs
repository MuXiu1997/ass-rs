//! Regression contract for empty override recovery without losing later tags.

use ass_core::analysis::events::tags::{
    parse_override_block, DiagnosticKind, OverrideTag, TagDiagnostic,
};

fn check(
    parse: impl for<'a> Fn(&'a str, usize, &mut Vec<OverrideTag<'a>>, &mut Vec<TagDiagnostic<'a>>),
) {
    for (input, expected, empty_count) in [
        (r"\\b1", vec![("b", "1", 12)], 1),
        (r"\b1\", vec![("b", "1", 11)], 1),
        (r"\\fnArial", vec![("fn", "Arial", 12)], 1),
        (
            r"\b1\\fnArial\i1",
            vec![("b", "1", 11), ("fn", "Arial", 15), ("i", "1", 23)],
            1,
        ),
        (r"\\\rOther", vec![("r", "Other", 13)], 2),
        (r"\\p1\q2", vec![("p", "1", 12), ("q", "2", 15)], 1),
    ] {
        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse(input, 11, &mut tags, &mut diagnostics);
        let actual: Vec<_> = tags
            .iter()
            .map(|t| (t.name(), t.args(), t.position()))
            .collect();
        assert_eq!(actual, expected, "{input:?}");
        assert_eq!(diagnostics.len(), empty_count, "{input:?}");
        assert!(diagnostics
            .iter()
            .all(|d| d.kind == DiagnosticKind::EmptyOverride));
        for diagnostic in diagnostics {
            assert!(input.is_char_boundary(diagnostic.offset - 11));
            assert!(input.contains(diagnostic.span));
        }
    }
}

#[test]
fn standard_empty_recovery_keeps_following_font_state_tags() {
    check(parse_override_block);
}

#[cfg(feature = "plugins")]
#[test]
fn registry_empty_recovery_keeps_following_font_state_tags() {
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

#[test]
fn malformed_unicode_after_backslash_has_bounded_diagnostics() {
    for input in [r"\字\b1", r"\🦀\fnArial", r"\\字\b1"] {
        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block(input, 0, &mut tags, &mut diagnostics);
        assert_eq!(tags.len(), 1);
        assert!(!diagnostics.is_empty());
    }
}

#[cfg(feature = "plugins")]
#[test]
fn registry_malformed_unicode_after_backslash_has_bounded_diagnostics() {
    for input in [r"\字\b1", r"\🦀\fnArial", r"\\字\b1"] {
        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        ass_core::analysis::events::tags::parse_override_block_with_registry(
            input,
            0,
            &mut tags,
            &mut diagnostics,
            None,
        );
        assert_eq!(tags.len(), 1);
        assert!(!diagnostics.is_empty());
    }
}

#[test]
fn surrounding_unicode_and_parenthesized_arguments_keep_their_boundaries() {
    for input in [
        r"注释\t(0,100,\fs40)\b1",
        r"注释\t(0,100,\clip(1,2,3,4))\b1",
        r"\t(0,100,\fs40",
    ] {
        let mut expected_tags = Vec::new();
        let mut expected_diagnostics = Vec::new();
        parse_override_block(input, 0, &mut expected_tags, &mut expected_diagnostics);
        assert_eq!(expected_tags[0].name(), "t");
        assert!(expected_diagnostics.is_empty());
        #[cfg(feature = "plugins")]
        {
            let mut tags = Vec::new();
            let mut diagnostics = Vec::new();
            ass_core::analysis::events::tags::parse_override_block_with_registry(
                input,
                0,
                &mut tags,
                &mut diagnostics,
                None,
            );
            assert_eq!(
                tags.iter()
                    .map(|t| (t.name(), t.args()))
                    .collect::<Vec<_>>(),
                expected_tags
                    .iter()
                    .map(|t| (t.name(), t.args()))
                    .collect::<Vec<_>>()
            );
            assert!(diagnostics.is_empty());
        }
    }
}
