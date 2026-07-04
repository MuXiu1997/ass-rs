//! Command to delete multiple events from an ASS document.

use super::helpers::collect_event_lines;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Command to delete multiple events from the ASS document
///
/// Removes multiple events (Dialogue or Comment) at the specified indices from the `[Events]` section.
/// Indices are automatically sorted and processed in reverse order to maintain correctness.
/// All indices are 0-based and include both Dialogue and Comment events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchDeleteEventsCommand {
    /// Indices of events to delete (will be sorted and deduplicated)
    pub event_indices: Vec<usize>,
}

impl BatchDeleteEventsCommand {
    /// Create a new batch delete events command
    pub fn new(event_indices: Vec<usize>) -> Self {
        Self { event_indices }
    }
}

impl EditorCommand for BatchDeleteEventsCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        if self.event_indices.is_empty() {
            return Ok(CommandResult::success());
        }

        // Sort indices in descending order to delete from end to start
        // This prevents index shifting issues
        let mut sorted_indices = self.event_indices.clone();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
        sorted_indices.dedup();

        let content = document.text();
        let event_positions = collect_event_lines(&content)?;

        // Delete events in reverse order to avoid index shifting
        let mut total_deleted = 0;
        let mut first_delete_pos = Position::new(content.len());

        for index in &sorted_indices {
            if let Some(event_line) = event_positions
                .iter()
                .find(|event_line| event_line.index == *index)
            {
                let range = Range::new(
                    Position::new(event_line.start),
                    Position::new(event_line.end_with_newline),
                );
                document.delete(range)?;
                total_deleted += 1;

                // Track the earliest deletion position
                if range.start.offset < first_delete_pos.offset {
                    first_delete_pos = range.start;
                }
            }
        }

        if total_deleted > 0 {
            Ok(CommandResult::success_with_change(
                Range::new(first_delete_pos, first_delete_pos),
                first_delete_pos,
            ))
        } else {
            Ok(CommandResult::success())
        }
    }

    fn description(&self) -> &str {
        "Delete multiple events"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.event_indices.len() * core::mem::size_of::<usize>()
    }
}
