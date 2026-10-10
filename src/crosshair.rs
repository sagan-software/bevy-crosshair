//! Validated console configuration and geometry entry point.

use crate::HitMarker;
use crate::SettingError;
use crate::Style;
use crate::WeaponState;
use crate::settings::Settings;
use bevy::prelude::*;
use std::fmt;
use std::str::FromStr;

/// Crosshair configuration stored on the entity that receives hit events. Text
/// updates
/// validate every value before modifying this component or its generated
/// geometry.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(opaque)]
#[require(CrosshairInput, HitMarker = HitMarker::inactive())]
pub struct Crosshair {
    /// Authoritative validated settings, shared by every rendering mode.
    pub(crate) settings: Settings,
}

impl Crosshair {
    /// Returns the configured style before spectator policy selects a displayed
    /// crosshair.
    /// Changing host spread, recoil, or weapon state never changes this stored
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::Crosshair;
    /// use bevy_crosshair::Style;
    /// assert_eq!(Crosshair::default().style(), Style::DynamicQuad);
    /// ```
    pub fn style(&self) -> Style {
        self.settings.style
    }

    /// Selects a style while retaining all other settings. This consumes and
    /// returns the
    /// component, making style selection convenient when constructing an entity
    /// bundle.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::Crosshair;
    /// use bevy_crosshair::Style;
    /// let crosshair = Crosshair::default().with_style(Style::StaticCross);
    /// ```
    pub fn with_style(mut self, style: Style) -> Self {
        self.settings.style = style;
        self
    }

    /// Reads one setting as canonical console text. The returned owned string
    /// contains only its
    /// value, without the setting name or trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`SettingError`] for an unsupported setting name.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::Crosshair;
    /// assert_eq!(Crosshair::default().get("cl_crosshair_length")?, "8");
    /// # Ok::<(), bevy_crosshair::SettingError>(())
    /// ```
    pub fn get(&self, name: &str) -> Result<String, SettingError> {
        self.settings.get(name)
    }

    /// Validates and replaces one setting. Failure leaves the old value intact.
    ///
    /// # Errors
    ///
    /// Returns [`SettingError`] for an unknown name or invalid value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_crosshair::Crosshair;
    /// let mut crosshair = Crosshair::default();
    /// crosshair.set("cl_crosshair_length", "12")?;
    /// # Ok::<(), bevy_crosshair::SettingError>(())
    /// ```
    pub fn set(&mut self, name: &str, value: &str) -> Result<(), SettingError> {
        // Resolve the current-client setting name before parsing its value.
        // Validate grammar and bounds before replacing the stored field.
        self.settings.set(name, value)
    }
}

impl FromStr for Crosshair {
    type Err = SettingError;
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        // Parse into a fresh default so an invalid assignment cannot partially update a caller.
        let mut crosshair = Self::default();
        for assignment in source
            .split([';', '\n'])
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            // Reject extra tokens before accepting an optional balanced pair of quotes.
            let mut tokens = assignment.split_whitespace();
            let (Some(name), Some(value), None) = (tokens.next(), tokens.next(), tokens.next())
            else {
                return Err(SettingError::Assignment(assignment.into()));
            };
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value);
            crosshair.set(name, value)?;
        }
        Ok(crosshair)
    }
}

impl fmt::Display for Crosshair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.settings.fmt(formatter)
    }
}

/// Input values in physical viewport pixels, with positive Y pointing down.
///
/// The host supplies spread and recoil from its own weapon simulation.
/// Physical measurements must be finite and at most 32768 pixels in magnitude.
/// Viewport dimensions must be positive; spread and shot feedback are
/// nonnegative.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct CrosshairInput {
    /// Physical viewport width and height in pixels. Both dimensions must be
    /// positive, finite,
    /// and at most 32768; invalid dimensions produce empty geometry.
    pub viewport: Vec2,
    /// Radial weapon inaccuracy in physical pixels, supplied by the host weapon
    /// simulation.
    /// Dynamic styles apply this distance after authored dimensions have been
    /// scaled.
    pub spread: f32,
    /// Additional radial gap after a shot, in physical pixels. Only the
    /// shot-feedback style uses
    /// this value; the host controls its decay between frames.
    pub shot: f32,
    /// Predicted aim-punch displacement in physical pixels, with positive Y
    /// pointing down. The
    /// crosshair applies this offset only when its recoil setting is enabled.
    pub recoil: Vec2,
    /// Mutually exclusive weapon state supplied by the host controller. Scoped
    /// weapons select
    /// scope geometry, while held grenades select their configured delayed
    /// alignment guides.
    pub weapon: WeaponState,
    /// Whether the host aim ray currently intersects a friendly actor. When the
    /// warning setting
    /// is enabled, this adds a separate warning above the crosshair.
    pub is_friendly: bool,
    /// Observed player and supplied configuration, or the local player. The local
    /// spectator
    /// settings decide whether to display this supplied configuration or a
    /// fallback.
    pub spectator: Spectator,
}

impl Default for CrosshairInput {
    fn default() -> Self {
        Self {
            viewport: Vec2::new(1920.0, 1080.0),
            spread: 0.0,
            shot: 0.0,
            recoil: Vec2::ZERO,
            weapon: WeaponState::Hip,
            is_friendly: false,
            spectator: Spectator::Local,
        }
    }
}

impl CrosshairInput {
    /// Rejects non-finite or negative physical measurements at the host boundary.
    pub(crate) fn is_valid(&self) -> bool {
        self.viewport.is_finite()
            && self.viewport.min_element() > 0.0
            && self.viewport.max_element() <= 32768.0
            && self.spread.is_finite()
            && (0.0..=32768.0).contains(&self.spread)
            && self.shot.is_finite()
            && (0.0..=32768.0).contains(&self.shot)
            && self.recoil.is_finite()
            && self.recoil.abs().max_element() <= 32768.0
    }
}

/// Spectator relationship used to select a crosshair configuration. The host
/// supplies
/// relationship and takeover state; the plugin never queries accounts or game
/// ownership.
#[derive(Clone, Debug, Default)]
pub enum Spectator {
    /// Local player's configuration, without any spectator override. Use this state
    /// for ordinary
    /// gameplay and whenever the host has no observed player to display.
    #[default]
    Local,
    /// Observed friend or party member's configuration, owned by this input. Both
    /// the friends-
    /// only and everyone observer policies permit this supplied configuration to
    /// display.
    Friend(Box<Crosshair>),
    /// Observed unrelated player's configuration, owned by this input. Only the
    /// everyone observer
    /// policy permits it; other policies retain the local player's settings.
    Player(Box<Crosshair>),
    /// Observed bot that can be taken over by the local player. The bot policy
    /// chooses local
    /// settings or current defaults for its crosshair.
    AvailableBot,
    /// Observed bot that cannot be taken over by the local player. The
    /// takeover-only policy falls
    /// back to current defaults while the own policy retains local settings.
    UnavailableBot,
}
