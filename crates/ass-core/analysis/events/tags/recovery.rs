//! UTF-8-safe recovery that leaves a following tag's backslash unconsumed.

use super::types::{DiagnosticKind, TagDiagnostic};

/// Records an empty item and advances only over a complete non-backslash character.
pub(super) fn empty_override<'a>(
    content: &'a str,
    start_pos: usize,
    tag_start: usize,
    byte_pos: &mut usize,
    char_pos: &mut usize,
    chars: &[char],
) -> TagDiagnostic<'a> {
    let skip = chars
        .get(*char_pos)
        .filter(|&&ch| ch != '\\')
        .map_or(0, |ch| ch.len_utf8());
    *byte_pos += skip;
    *char_pos += usize::from(skip > 0);
    TagDiagnostic {
        span: &content[tag_start..*byte_pos],
        offset: start_pos + tag_start,
        kind: DiagnosticKind::EmptyOverride,
    }
}
