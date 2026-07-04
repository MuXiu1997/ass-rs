//! Command to toggle event type between Dialogue and Comment.

use super::helpers::collect_event_lines;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Command to toggle event type between Dialogue and Comment
#[derive(Debug, Clone)]
pub struct ToggleEventTypeCommand {
    pub event_indices: Vec<usize>,
    pub description: Option<String>,
}

impl ToggleEventTypeCommand {
    /// Create a new event type toggle command
    pub fn new(event_indices: Vec<usize>) -> Self {
        Self {
            event_indices,
            description: None,
        }
    }

    /// Toggle a single event
    pub fn single(event_index: usize) -> Self {
        Self::new(vec![event_index])
    }

    /// Toggle all events
    pub fn all() -> Self {
        Self::new(Vec::new()) // Empty means all events
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for ToggleEventTypeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let event_lines = collect_event_lines(&content)?;
        let mut replacements = Vec::new();
        let mut total_range: Option<Range> = None;

        for event_line in event_lines {
            let should_toggle =
                self.event_indices.is_empty() || self.event_indices.contains(&event_line.index);

            if !should_toggle {
                continue;
            }

            let new_line = if event_line.line.starts_with("Dialogue:") {
                event_line.line.replacen("Dialogue:", "Comment:", 1)
            } else {
                event_line.line.replacen("Comment:", "Dialogue:", 1)
            };

            let change_range = Range::new(
                Position::new(event_line.start),
                Position::new(event_line.start + new_line.len()),
            );
            total_range = Some(match total_range {
                Some(existing) => existing.union(&change_range),
                None => change_range,
            });
            replacements.push((event_line.start, event_line.end, new_line));
        }

        for (start, end, new_line) in replacements.iter().rev() {
            let range = Range::new(Position::new(*start), Position::new(*end));
            document.replace(range, new_line)?;
        }

        let changes_made = replacements.len();

        if changes_made > 0 {
            Ok(CommandResult::success_with_change(
                total_range.unwrap_or(Range::new(Position::new(0), Position::new(0))),
                Position::new(document.len_bytes()),
            )
            .with_message(format!("Toggled type for {changes_made} events")))
        } else {
            Ok(CommandResult::success().with_message("No events were toggled".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Toggle event type")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.event_indices.len() * core::mem::size_of::<usize>()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
