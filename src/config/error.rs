//! Configuration failures shared by the text boundary and closed style
//! vocabulary.

/// Configuration failure with the offending input preserved for diagnostics.
/// Unknown names,
/// malformed assignments, and invalid values remain distinct categories so
/// callers can report
/// specific corrections.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SettingError {
    /// The setting name is not part of the selected CS2 profile. The payload
    /// preserves the
    /// supplied name, including any unsupported alias or legacy spelling.
    #[error("unsupported crosshair setting: {0}{hint}", hint = replacement_hint(.0))]
    Unknown(String),
    /// An assignment does not contain exactly one name and value. The payload
    /// preserves the
    /// trimmed assignment so an editor can identify the malformed input.
    #[error("expected one setting name and value: {0}")]
    Assignment(String),
    /// A value does not satisfy its setting's grammar or bounds. Direct updates
    /// return this error
    /// before changing the previously validated value stored in the component.
    #[error("invalid value {value:?} for {name}")]
    Value {
        /// Setting whose value failed validation, preserved as supplied by the caller.
        /// This
        /// identifies the specific control whose grammar or numeric bounds rejected the
        /// update.
        name: String,
        /// Original value before parsing, preserved for diagnostics. Configuration
        /// parsing removes a
        /// balanced pair of outer quotes before validating the value and constructing
        /// this error.
        value: String,
    },
}

/// Names the active setting when a removed console spelling has a direct
/// successor.
fn replacement_hint(name: &str) -> &'static str {
    let aliases = [
        ("cl_crosshairalpha", "; use cl_crosshaircolor_a"),
        ("cl_crosshairsize", "; use cl_crosshair_length"),
        ("cl_crosshairthickness", "; use cl_crosshair_thickness"),
    ];
    aliases
        .into_iter()
        .find_map(|(legacy, hint)| (name == legacy).then_some(hint))
        .unwrap_or("")
}
