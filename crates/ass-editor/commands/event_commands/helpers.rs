//! Shared parsing helpers for event management commands.
//!
//! Provides robust ASS event line parsing with proper comma handling for the
//! `Effect` and `Text` fields, used by the sibling command submodules.

use crate::core::EditorError;
use ass_core::parser::ast::{Event, EventType, Span};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Physical source range for a parsed event command target.
#[derive(Debug, Clone, Copy)]
pub(super) struct EventLine<'a> {
    pub index: usize,
    pub start: usize,
    pub end: usize,
    pub end_with_newline: usize,
    pub line: &'a str,
}

/// Collect event lines from the `[Events]` section, skipping blank/comment lines.
pub(super) fn collect_event_lines(
    content: &str,
) -> core::result::Result<Vec<EventLine<'_>>, EditorError> {
    let events_start = content
        .find("[Events]")
        .ok_or_else(|| EditorError::command_failed("Events section not found"))?;

    let events_content = &content[events_start..];
    let format_line_end = events_content
        .find("Format:")
        .and_then(|format_pos| {
            events_content[format_pos..]
                .find('\n')
                .map(|newline_pos| events_start + format_pos + newline_pos + 1)
        })
        .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

    let mut event_lines = Vec::new();
    let mut current_index = 0;
    let mut event_start = format_line_end;

    while event_start < content.len() {
        let newline_pos = content[event_start..].find('\n');
        let line_end = newline_pos.map_or(content.len(), |pos| event_start + pos);
        let end_with_newline = newline_pos.map_or(content.len(), |_| line_end + 1);
        let line = &content[event_start..line_end];
        let trimmed = line.trim_start();

        if trimmed.starts_with('[') {
            break;
        }

        if line.trim().is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            event_start = end_with_newline;
            continue;
        }

        if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
            event_lines.push(EventLine {
                index: current_index,
                start: event_start,
                end: line_end,
                end_with_newline,
                line,
            });
            current_index += 1;
        }

        event_start = end_with_newline;
    }

    Ok(event_lines)
}

/// Helper function to parse an ASS event line with proper comma handling
/// Returns parsed Event struct or error if parsing fails
pub(super) fn parse_event_line(line: &str) -> core::result::Result<Event<'_>, EditorError> {
    // Extract event type
    let colon_pos = line
        .find(':')
        .ok_or_else(|| EditorError::command_failed("Invalid event format: missing colon"))?;
    let event_type_str = &line[..colon_pos];
    let fields_part = line[colon_pos + 1..].trim();

    let event_type = match event_type_str {
        "Dialogue" => EventType::Dialogue,
        "Comment" => EventType::Comment,
        _ => return Err(EditorError::command_failed("Unknown event type")),
    };

    // Parse fields carefully - Effect field can contain commas, so we need special handling
    // Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    let parts: Vec<&str> = fields_part.splitn(10, ',').collect();
    if parts.len() < 10 {
        return Err(EditorError::command_failed(
            "Invalid event format: insufficient fields",
        ));
    }

    // The issue is that parts[8] (Effect) and parts[9] (Text) might be incorrectly split
    // if Effect contains commas. We need to rejoin them properly.

    // First 8 fields are safe (no commas expected)
    let layer = parts[0].trim();
    let start = parts[1].trim();
    let end = parts[2].trim();
    let style = parts[3].trim();
    let name = parts[4].trim();
    let margin_l = parts[5].trim();
    let margin_r = parts[6].trim();
    let margin_v = parts[7].trim();

    // For Effect and Text, we need to find the correct comma that separates them
    // Effect can contain commas, but Text is the final field

    // Calculate where effect+text starts in the original string
    let prefix_len = parts[0..8].iter().map(|s| s.len()).sum::<usize>() + 8; // +8 for commas
    let remaining = &fields_part[prefix_len..];

    // Find the last comma that's not inside parentheses
    let mut split_point = None;
    let chars: Vec<char> = remaining.chars().collect();
    let mut paren_depth = 0;

    for (i, &ch) in chars.iter().enumerate().rev() {
        match ch {
            ')' => paren_depth += 1,
            '(' => paren_depth -= 1,
            ',' if paren_depth == 0 => {
                split_point = Some(i);
                break;
            }
            _ => {}
        }
    }

    let (effect, text) = if let Some(split) = split_point {
        let effect_part = remaining[..split].trim();
        let text_part = remaining[split + 1..].trim();
        (effect_part, text_part)
    } else {
        // No comma found outside parentheses, treat entire remaining as effect
        (remaining.trim(), "")
    };

    Ok(Event {
        event_type,
        layer,
        start,
        end,
        style,
        name,
        margin_l,
        margin_r,
        margin_v,
        margin_t: None,
        margin_b: None,
        effect,
        text,
        span: Span::new(0, line.len(), 1, 1), // Dummy span
    })
}
