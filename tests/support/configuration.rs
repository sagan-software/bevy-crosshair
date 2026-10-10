//! Shared configuration conformance through the public API.

use super::*;

/// Covers text ingress, export, failure formatting, and each current setting.
fn defaults() {
    // Start each case from the published defaults to detect unintended normalization.
    let original = Crosshair::default();
    assert_eq!(original.style(), Style::DynamicQuad);
    // Export must round-trip every setting and match an empty configuration.
    let text = original.to_string();
    assert_eq!(
        text.parse::<Crosshair>()
            .expect("the fixture supplies a valid value")
            .to_string(),
        text
    );
    assert_eq!(
        " \n; "
            .parse::<Crosshair>()
            .expect("the fixture supplies a valid value")
            .to_string(),
        text
    );
    text.lines().for_each(|line| check_default(line, &text));
}

/// Checks the channels boundary independently.
fn channels() {
    // Start each case from the published defaults to detect unintended normalization.
    let original = Crosshair::default();
    for channel in [
        "cl_crosshaircolor_r",
        "cl_crosshaircolor_g",
        "cl_crosshaircolor_b",
        "cl_crosshaircolor_a",
        "cl_crosshairoutline_r",
        "cl_crosshairoutline_g",
        "cl_crosshairoutline_b",
        "cl_crosshairoutline_a",
    ] {
        // Both channel endpoints are valid; each adjacent or fractional value fails.
        for is_valid in ["0", "255"] {
            original
                .clone()
                .set(channel, is_valid)
                .expect("the fixture supplies a valid value");
        }
        ["-1", "256", "2.5", "NaN"].into_iter().for_each(|invalid| {
            original
                .clone()
                .set(channel, invalid)
                .expect_err("invalid input must be rejected");
        });
    }
}

/// Checks the numbers boundary independently.
fn numbers() {
    let bounds = [
        ("cl_crosshair_length", 0.0, 255.0),
        ("cl_crosshair_thickness", 0.0, 32.0),
        ("cl_crosshair_gap", -3840.0, 3840.0),
        ("cl_crosshair_screen_height", 240.0, 32768.0),
        ("cl_crosshair_dynamic_spread_limit", 0.0, 255.0),
        ("cl_crosshair_dynamic_splitdist", 0.0, 127.0),
        ("cl_crosshair_dynamic_maxdist_splitratio", 0.0, 1.0),
        ("cl_crosshair_dynamic_splitalpha_innermod", 0.0, 1.0),
        ("cl_crosshair_dynamic_splitalpha_outermod", 0.3, 1.0),
        ("cl_crosshair_sniper_width", 1.0, 6.0),
        ("cl_ironsight_dot_scale", 0.1, 2.0),
        ("cl_grenadecrosshairdelay_flash", 0.0, 5.0),
        ("cl_grenadecrosshairdelay_explosive", 0.0, 5.0),
        ("cl_grenadecrosshairdelay_fire", 0.0, 5.0),
        ("cl_grenadecrosshairdelay_smoke", 0.0, 5.0),
        ("cl_grenadecrosshairdelay_decoy", 0.0, 5.0),
    ];
    bounds
        .into_iter()
        .for_each(|(name, minimum, maximum)| check_number(name, minimum, maximum));
}

/// Checks the vocabularies boundary independently.
fn vocabularies() {
    // Start each case from the published defaults to detect unintended normalization.
    let original = Crosshair::default();
    for name in [
        "cl_crosshair_drawoutline",
        "cl_show_observer_crosshair",
        "cl_observed_bot_crosshair",
    ] {
        // The three legal codes are independent of their textual field names.
        for value in ["0", "1", "2"] {
            original
                .clone()
                .set(name, value)
                .expect("the fixture supplies a valid value");
        }
        ["3", "-1", "no"].into_iter().for_each(|value| {
            original
                .clone()
                .set(name, value)
                .expect_err("invalid input must be rejected");
        });
    }
}

/// Checks the styles boundary independently.
fn styles() {
    // Numeric style codes have exact canonical spellings.
    (0..10).for_each(|value| {
        assert_eq!(
            Style::try_from(value)
                .expect("the fixture supplies a valid value")
                .to_string(),
            value.to_string()
        );
    });
    // Unknown codes and malformed numbers share the typed value error.
    ["10", "-1", "NaN", "256"].into_iter().for_each(|invalid| {
        invalid
            .parse::<Style>()
            .expect_err("invalid input must be rejected");
    });
}

/// Checks the assignments boundary independently.
fn assignments() {
    // Start each case from the published defaults to detect unintended normalization.
    let original = Crosshair::default();
    for source in [
        "missing",
        "cl_crosshair_length",
        "cl_crosshair_length 1 2",
        "cl_crosshair_length \"2",
        "unknown 2",
    ] {
        let error = source
            .parse::<Crosshair>()
            .expect_err("the fixture must be rejected");
        assert!(!error.to_string().is_empty());
        assert_eq!(error.clone(), error);
    }
    // Direct lookup must reject unknown names too.
    original
        .get("unknown")
        .expect_err("invalid input must be rejected");
}

/// Checks the legacy names boundary independently.
fn legacy_names() {
    // Start each case from the published defaults to detect unintended normalization.
    let original = Crosshair::default();
    [
        ("cl_crosshairalpha", "cl_crosshaircolor_a"),
        ("cl_crosshairsize", "cl_crosshair_length"),
        ("cl_crosshairthickness", "cl_crosshair_thickness"),
    ]
    .into_iter()
    .for_each(|(removed, replacement)| {
        assert!(
            original
                .get(removed)
                .expect_err("the fixture must be rejected")
                .to_string()
                .contains(replacement)
        );
    });
    // Later assignments replace earlier values after quote parsing.
    let duplicate: Crosshair = "cl_crosshair_length 1;cl_crosshair_length \"12\""
        .parse()
        .expect("the fixture supplies a valid value");
    assert_eq!(
        duplicate
            .get("cl_crosshair_length")
            .expect("the fixture supplies a valid value"),
        "12"
    );
}

/// Covers every text boundary through the public configuration API.
pub(crate) fn run() {
    // Defaults and numeric fields exercise ingress and canonical export together.
    defaults();
    channels();
    numbers();
    // Closed vocabularies and malformed assignments retain their distinct errors.
    vocabularies();
    styles();
    assignments();
    legacy_names();
}

/// Checks one exported setting without sharing mutable state with another case.
fn check_default(line: &str, canonical: &str) {
    // A rejected update must preserve every previously exported value.
    let (name, value) = line.split_once(' ').expect("export emits a name and value");
    let mut changed = Crosshair::default();
    assert_eq!(changed.get(name).expect("exported name exists"), value);
    changed.set(name, value).expect("exported value is valid");
    changed
        .set(name, "invalid")
        .expect_err("invalid text is rejected");
    assert_eq!(changed.to_string(), canonical);
    // Boolean aliases normalize at ingress while numeric values keep their own grammar.
    if value == "true" || value == "false" {
        ["true", "false", "1", "0"]
            .into_iter()
            .for_each(|spelling| {
                changed
                    .set(name, spelling)
                    .expect("boolean spelling is accepted");
            });
        changed.set(name, "2").expect_err("two is not a boolean");
    }
}

/// Exercises both endpoints and rejected numeric neighbors for one field.
fn check_number(name: &str, minimum: f32, maximum: f32) {
    // Start each case from the published defaults to detect unintended normalization.
    let original = Crosshair::default();
    for value in [minimum, maximum] {
        original
            .clone()
            .set(name, &value.to_string())
            .expect("the fixture supplies a valid value");
    }
    // Non-finite and below-minimum values must fail without changing the original.
    [
        "NaN".to_owned(),
        "inf".to_owned(),
        (minimum - 0.1).to_string(),
    ]
    .into_iter()
    .for_each(|value| {
        original
            .clone()
            .set(name, &value)
            .expect_err("invalid input must be rejected");
    });
    if name != "cl_crosshair_screen_height" {
        original
            .clone()
            .set(name, &(maximum + 0.1).to_string())
            .expect_err("above-maximum input fails");
    }
}
