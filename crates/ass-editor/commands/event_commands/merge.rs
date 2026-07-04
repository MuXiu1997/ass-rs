//! Command to merge two consecutive events into one.

use super::helpers::{collect_event_lines, parse_event_line};
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Command to merge two consecutive events
#[derive(Debug, Clone)]
pub struct MergeEventsCommand {
    pub first_event_index: usize,
    pub second_event_index: usize,
    pub merge_text_separator: String, // Text to put between merged texts
    pub description: Option<String>,
}

impl MergeEventsCommand {
    /// Create a new event merge command
    pub fn new(first_event_index: usize, second_event_index: usize) -> Self {
        Self {
            first_event_index,
            second_event_index,
            merge_text_separator: " ".to_string(), // Default space separator
            description: None,
        }
    }

    /// Set the text separator for merged text
    pub fn with_separator(mut self, separator: String) -> Self {
        self.merge_text_separator = separator;
        self
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for MergeEventsCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        if self.first_event_index >= self.second_event_index {
            return Err(EditorError::command_failed(
                "First event index must be less than second event index",
            ));
        }

        let content = document.text();
        let event_lines = collect_event_lines(&content)?;
        let events: Vec<_> = event_lines
            .into_iter()
            .filter(|event_line| {
                event_line.index == self.first_event_index
                    || event_line.index == self.second_event_index
            })
            .collect();

        if events.len() != 2 {
            return Err(EditorError::command_failed(
                "Could not find both events to merge",
            ));
        }

        // Parse both events using ass-core's parser
        let first_event_line = events[0].line;
        let second_event_line = events[1].line;

        let first_event = parse_event_line(first_event_line)?;
        let second_event = parse_event_line(second_event_line)?;

        // Use first event's properties, second event's end time, merged text
        let merged_text = format!(
            "{}{}{}",
            first_event.text, self.merge_text_separator, second_event.text
        );

        // Create merged event
        let event_type_str = match first_event.event_type {
            EventType::Dialogue => "Dialogue",
            EventType::Comment => "Comment",
            _ => "Dialogue", // Default fallback
        };
        let merged_event = format!(
            "{}: {},{},{},{},{},{},{},{},{},{}",
            event_type_str,
            first_event.layer,
            first_event.start,
            second_event.end,
            first_event.style,
            first_event.name,
            first_event.margin_l,
            first_event.margin_r,
            first_event.margin_v,
            first_event.effect,
            merged_text
        );

        // Replace both events with the merged one
        let first_start = events[0].start;
        let second_end = events[1].end_with_newline;
        let range = Range::new(Position::new(first_start), Position::new(second_end));
        let replacement = format!("{merged_event}\n");

        document.replace(range, &replacement)?;

        let end_pos = Position::new(first_start + replacement.len());
        Ok(CommandResult::success_with_change(
            Range::new(Position::new(first_start), end_pos),
            end_pos,
        )
        .with_message(format!(
            "Merged events {} and {}",
            self.first_event_index, self.second_event_index
        )))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Merge events")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.merge_text_separator.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
