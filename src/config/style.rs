//! Current CS2 style vocabulary, preserving its numeric identifiers.

use std::str::FromStr;
use strum::VariantArray;

/// Crosshair styles from CS2 build 1.41.9.0, preserving numeric console
/// identifiers. Parse or
/// convert a supported identifier before storing a style in a Crosshair
/// component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::Display, strum::VariantArray)]
#[repr(u8)]
pub enum Style {
    /// Cross with supplied spread feedback. Four bars move outward as weapon
    /// inaccuracy
    /// increases; configured length and thickness remain independent of that radial
    /// motion.
    #[strum(serialize = "0")]
    DynamicCross = 0,
    /// Circle with supplied spread feedback. Weapon inaccuracy changes the ring
    /// radius while
    /// authored thickness, outline mode, and independently configured colors retain
    /// their
    /// selected values.
    #[strum(serialize = "1")]
    DynamicCircle = 1,
    /// Cross whose bars split above the configured spread threshold. Inner and
    /// outer portions use
    /// independent opacity multipliers and divide length according to the split
    /// ratio.
    #[strum(serialize = "2")]
    Legacy = 2,
    /// Circle unaffected by weapon spread or shot feedback. Configured gap
    /// determines its radius
    /// while recoil can still move its center when following recoil is enabled.
    #[strum(serialize = "3")]
    StaticCircle = 3,
    /// Cross unaffected by weapon spread or shot feedback. Configured gap remains
    /// fixed while
    /// recoil can still move its center when following recoil is enabled.
    #[strum(serialize = "4")]
    StaticCross = 4,
    /// Cross affected by shot feedback only. The host supplies additional radial
    /// gap after firing
    /// and controls its decay; ordinary weapon spread does not move these bars.
    #[strum(serialize = "5")]
    ShotCross = 5,
    /// Center dot without outer bars or rings. Authored thickness determines its
    /// dimensions, and
    /// the configured outline surrounds it even when the separate dot flag is
    /// disabled.
    #[strum(serialize = "6")]
    Dot = 6,
    /// Four curved segments with supplied spread feedback. Their radius increases
    /// with weapon
    /// inaccuracy, and T shape removes the upper segment while retaining the
    /// remaining three.
    #[default]
    #[strum(serialize = "7")]
    DynamicQuad = 7,
    /// Hollow square unaffected by weapon spread or shot feedback. Gap and
    /// thickness determine
    /// its dimensions, while configured colors and recoil behavior remain
    /// available.
    #[strum(serialize = "8")]
    StaticSquare = 8,
    /// Four curved segments with a configurable angular split. The split ratio
    /// controls angular
    /// coverage, while weapon spread and shot feedback leave the radius unchanged.
    #[strum(serialize = "9")]
    StaticQuad = 9,
}

impl TryFrom<u8> for Style {
    type Error = crate::SettingError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::VARIANTS
            .get(usize::from(value))
            .copied()
            .ok_or_else(|| crate::SettingError::Value {
                name: "cl_crosshairstyle".into(),
                value: value.to_string(),
            })
    }
}

impl FromStr for Style {
    type Err = crate::SettingError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let number = value
            .parse::<u8>()
            .map_err(|_error| crate::SettingError::Value {
                name: "cl_crosshairstyle".into(),
                value: value.into(),
            })?;
        Self::try_from(number)
    }
}
