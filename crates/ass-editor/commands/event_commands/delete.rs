//! Command to delete a single event from an ASS document.

use super::helpers::collect_event_lines;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::format;

/// Command to delete a single event from the ASS document
///
/// Removes an event (Dialogue or Comment) at the specified index from the `[Events]` section.
/// The index is 0-based and includes both Dialogue and Comment events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteEventCommand {
    /// Index of the event to delete
    pub event_index: usize,
}

impl DeleteEventCommand {
    /// Create a new delete event command
    pub fn new(event_index: usize) -> Self {
        Self { event_index }
    }
}

impl EditorCommand for DeleteEventCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let delete_range = collect_event_lines(&content)?
            .into_iter()
            .find_map(|event_line| {
                (event_line.index == self.event_index).then(|| {
                    Range::new(
                        Position::new(event_line.start),
                        Position::new(event_line.end_with_newline),
                    )
                })
            });

        if let Some(range) = delete_range {
            document.delete(range)?;
            Ok(CommandResult::success_with_change(
                Range::new(range.start, range.start),
                range.start,
            ))
        } else {
            Err(EditorError::command_failed(format!(
                "Event index {} not found",
                self.event_index
            )))
        }
    }

    fn description(&self) -> &str {
        "Delete event"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
