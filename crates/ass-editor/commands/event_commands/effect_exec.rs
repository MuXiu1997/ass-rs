//! `EditorCommand` implementation for `EventEffectCommand`.

use super::effect::{EffectOperation, EventEffectCommand};
use super::helpers::{collect_event_lines, parse_event_line};
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

impl EditorCommand for EventEffectCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let event_lines = collect_event_lines(&content)?;
        let mut replacements = Vec::new();
        let mut total_range: Option<Range> = None;

        for event_line in event_lines {
            let should_modify =
                self.event_indices.is_empty() || self.event_indices.contains(&event_line.index);

            if !should_modify {
                continue;
            }

            if let Ok(event) = parse_event_line(event_line.line) {
                let new_effect = match self.operation {
                    EffectOperation::Set => self.effect.clone(),
                    EffectOperation::Clear => String::new(),
                    EffectOperation::Append => {
                        if event.effect.is_empty() {
                            self.effect.clone()
                        } else {
                            format!("{} {}", event.effect, self.effect)
                        }
                    }
                    EffectOperation::Prepend => {
                        if event.effect.is_empty() {
                            self.effect.clone()
                        } else {
                            format!("{} {}", self.effect, event.effect)
                        }
                    }
                };

                let event_type_str = match event.event_type {
                    EventType::Dialogue => "Dialogue",
                    EventType::Comment => "Comment",
                    _ => "Dialogue",
                };
                let new_line = format!(
                    "{}: {},{},{},{},{},{},{},{},{},{}",
                    event_type_str,
                    event.layer,
                    event.start,
                    event.end,
                    event.style,
                    event.name,
                    event.margin_l,
                    event.margin_r,
                    event.margin_v,
                    new_effect,
                    event.text
                );

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
        }

        for (start, end, new_line) in replacements.iter().rev() {
            let range = Range::new(Position::new(*start), Position::new(*end));
            document.replace(range, new_line)?;
        }

        let changes_made = replacements.len();

        if changes_made > 0 {
            let operation_name = match self.operation {
                EffectOperation::Set => "set",
                EffectOperation::Clear => "cleared",
                EffectOperation::Append => "appended",
                EffectOperation::Prepend => "prepended",
            };

            Ok(CommandResult::success_with_change(
                total_range.unwrap_or(Range::new(Position::new(0), Position::new(0))),
                Position::new(document.len_bytes()),
            )
            .with_message(format!("Effect {operation_name} for {changes_made} events")))
        } else {
            Ok(CommandResult::success().with_message("No events were modified".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Modify event effect")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.event_indices.len() * core::mem::size_of::<usize>()
            + self.effect.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
