//! Command to split a single event into two at a specific time.

use super::helpers::{collect_event_lines, parse_event_line};
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};
use ass_core::parser::ast::EventType;
use ass_core::utils::parse_ass_time;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// Command to split an event at a specific time
#[derive(Debug, Clone)]
pub struct SplitEventCommand {
    pub event_index: usize,
    pub split_time: String, // Time in ASS format (H:MM:SS.CC)
    pub description: Option<String>,
}

impl SplitEventCommand {
    /// Create a new event split command
    pub fn new(event_index: usize, split_time: String) -> Self {
        Self {
            event_index,
            split_time,
            description: None,
        }
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for SplitEventCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        // Parse split time to validate it
        let split_time_cs = parse_ass_time(&self.split_time).map_err(|_| {
            EditorError::command_failed(format!("Invalid time format: {}", self.split_time))
        })?;

        let content = document.text();
        let event_lines = collect_event_lines(&content)?;

        if let Some(event_line) = event_lines
            .into_iter()
            .find(|event_line| event_line.index == self.event_index)
        {
            let event = parse_event_line(event_line.line)?;

            // Validate split time is within event bounds
            let start_time_cs = event
                .start_time_cs()
                .map_err(|_| EditorError::command_failed("Invalid start time in event"))?;
            let end_time_cs = event
                .end_time_cs()
                .map_err(|_| EditorError::command_failed("Invalid end time in event"))?;

            if split_time_cs <= start_time_cs || split_time_cs >= end_time_cs {
                return Err(EditorError::command_failed(
                    "Split time must be between event start and end times",
                ));
            }

            // Create two new events
            let event_type_str = match event.event_type {
                EventType::Dialogue => "Dialogue",
                EventType::Comment => "Comment",
                _ => "Dialogue", // Default fallback
            };
            let first_event = format!(
                "{}: {},{},{},{},{},{},{},{},{},{}",
                event_type_str,
                event.layer,
                event.start,
                self.split_time,
                event.style,
                event.name,
                event.margin_l,
                event.margin_r,
                event.margin_v,
                event.effect,
                event.text
            );

            let second_event = format!(
                "{}: {},{},{},{},{},{},{},{},{},{}",
                event_type_str,
                event.layer,
                self.split_time,
                event.end,
                event.style,
                event.name,
                event.margin_l,
                event.margin_r,
                event.margin_v,
                event.effect,
                event.text
            );

            // Replace the original event with the two new events
            let replacement = format!("{first_event}\n{second_event}");
            let range = Range::new(
                Position::new(event_line.start),
                Position::new(event_line.end),
            );
            document.replace(range, &replacement)?;

            let end_pos = Position::new(event_line.start + replacement.len());
            return Ok(CommandResult::success_with_change(
                Range::new(Position::new(event_line.start), end_pos),
                end_pos,
            )
            .with_message(format!(
                "Split event {} at time {}",
                self.event_index, self.split_time
            )));
        }

        Err(EditorError::command_failed(format!(
            "Event index {} not found",
            self.event_index
        )))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Split event")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.split_time.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
