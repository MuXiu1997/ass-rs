//! Color and alpha names end before their unwrapped hexadecimal arguments.

/// Shortens an ASCII-scanned name to its libass color/alpha prefix.
///
/// `clip` takes precedence over `c`. A registry-aware caller first gives an
/// exact registered extension name precedence over built-in interpretation.
pub(super) fn split_color_name(
    content: &str,
    name_start: usize,
    char_pos: &mut usize,
    byte_pos: &mut usize,
) {
    let name = &content[name_start..*byte_pos];
    if name.starts_with("clip") {
        return;
    }
    if let Some(prefix) = ["alpha", "1c", "2c", "3c", "4c", "1a", "2a", "3a", "4a", "c"]
        .into_iter()
        .find(|prefix| name.starts_with(prefix))
    {
        *char_pos -= name.len() - prefix.len();
        *byte_pos = name_start + prefix.len();
    }
}
