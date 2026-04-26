//! Shared utility functions for the TUI

/// Convert a kebab-case engine/workflow ID to title case.
///
/// Example: `"gene-keys"` → `"Gene Keys"`
pub fn format_name(id: &str) -> String {
    id.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
