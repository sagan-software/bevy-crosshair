//! Validated current-client settings, defined once for parsing and export.

use crate::SettingError;
use crate::Style;
use bevy::color::LinearRgba;
use bevy::color::Srgba;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// Converts one setting at the text boundary, retaining its typed value.
trait Console: Sized + fmt::Display {
    /// Validates the complete value and setting-specific limits.
    fn read(source: &str, min: f32, max: f32) -> Option<Self>;
}

impl Console for f32 {
    fn read(source: &str, min: f32, max: f32) -> Option<Self> {
        let value: Self = source.parse().ok()?;
        (value.is_finite() && (min..=max).contains(&value)).then_some(value)
    }
}

impl Console for bool {
    fn read(source: &str, _min: f32, _max: f32) -> Option<Self> {
        match source {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }
    }
}

/// Color channel with the full byte range and no other numeric meaning.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Channel(u8);

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Console for Channel {
    fn read(source: &str, _min: f32, _max: f32) -> Option<Self> {
        source.parse().ok().map(Self)
    }
}

/// Generates a closed numeric console vocabulary and its standard conversions.
macro_rules! vocabulary {
    ($name:ident { $($(#[$doc:meta])* $variant:ident = $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum $name { $($(#[$doc])* $variant),+ }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self { $(Self::$variant => { let value = $value; write!(f, "{value}") }),+ }
            }
        }
        impl FromStr for $name {
            type Err = ();
            fn from_str(source: &str) -> Result<Self, Self::Err> {
                match source.parse::<u8>().map_err(|_| ())? { $($value => Ok(Self::$variant),)+ _ => Err(()) }
            }
        }
        impl Console for $name {
            fn read(source: &str, _min: f32, _max: f32) -> Option<Self> { source.parse().ok() }
        }
    };
}

// Match the numeric vocabularies captured from CS2 1.41.9.0.
vocabulary!(Outline {
    /// No outline.
    None = 0,
    /// One pixel on every side.
    Full = 1,
    /// Half-outline shading.
    Half = 2,
});
vocabulary!(ObserverPolicy {
    /// Always use local settings.
    Off = 0,
    /// Use settings from friends and party members.
    Friends = 1,
    /// Use any observed player's settings.
    Everyone = 2,
});
vocabulary!(BotPolicy {
    /// Use the local configuration.
    Own = 0,
    /// Use local configuration only when takeover is available.
    Takeover = 1,
    /// Use the default configuration.
    Default = 2,
});

impl Console for Style {
    fn read(source: &str, _min: f32, _max: f32) -> Option<Self> {
        source.parse().ok()
    }
}

/// Time value with console seconds at ingress and Duration internally.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Delay(Duration);

impl fmt::Display for Delay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_secs_f32().fmt(f)
    }
}

impl Console for Delay {
    fn read(source: &str, min: f32, max: f32) -> Option<Self> {
        f32::read(source, min, max).map(|seconds| Self(Duration::from_secs_f32(seconds)))
    }
}

/// Defines each field together with its single parsing and export rule.
macro_rules! settings {
    ($($(#[$doc:meta])* $field:ident: $ty:ty = $default:expr => ($name:literal, $min:expr, $max:expr);)+) => {
        /// Validated settings. Construction and updates occur only at the text
        /// boundary.
        #[derive(Clone, Debug)]
        pub(crate) struct Settings { $($(#[$doc])* pub(super) $field: $ty,)+ }
        impl Default for Settings {
            fn default() -> Self { Self { $($field: $default,)+ } }
        }
        impl fmt::Display for Settings {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                $(let name = $name;
                let value = &self.$field;
                writeln!(formatter, "{name} {value}")?;)+
                Ok(())
            }
        }
        impl Settings {
            /// Returns the setting in canonical console spelling.
            pub(super) fn get(&self, name: &str) -> Result<String, SettingError> {
                match name { $($name => Ok(self.$field.to_string()),)+ _ => Err(SettingError::Unknown(name.into())) }
            }
            /// Validates first, then replaces exactly one typed field.
            pub(super) fn set(&mut self, name: &str, value: &str) -> Result<(), SettingError> {
                match name {
                    $($name => {
                        self.$field = <$ty as Console>::read(value, $min, $max)
                            .ok_or_else(|| SettingError::Value { name: name.into(), value: value.into() })?;
                    },)+
                    _ => return Err(SettingError::Unknown(name.into())),
                }
                Ok(())
            }
        }
    };
}

settings! {
    /// Whether the crosshair is visible.
    is_enabled: bool = true => ("crosshair", 0.0, 1.0);
    /// Current-client shape and feedback mode.
    style: Style = Style::DynamicQuad => ("cl_crosshairstyle", 0.0, 9.0);
    /// Bar length in authored pixels.
    length: f32 = 8.0 => ("cl_crosshair_length", 0.0, 255.0);
    /// Bar thickness in authored pixels.
    thickness: f32 = 2.0 => ("cl_crosshair_thickness", 0.0, 32.0);
    /// Radial offset in authored pixels.
    gap: f32 = 4.0 => ("cl_crosshair_gap", -3840.0, 3840.0);
    /// Physical screen height used when authoring dimensions.
    height: f32 = 1080.0 => ("cl_crosshair_screen_height", 240.0, f32::MAX);
    /// Outline shape.
    outline: Outline = Outline::Full => ("cl_crosshair_drawoutline", 0.0, 2.0);
    /// Color r channel in sRGB byte units.
    color_r: Channel = Channel(0) => ("cl_crosshaircolor_r", 0.0, 255.0);
    /// Color g channel in sRGB byte units.
    color_g: Channel = Channel(255) => ("cl_crosshaircolor_g", 0.0, 255.0);
    /// Color b channel in sRGB byte units.
    color_b: Channel = Channel(0) => ("cl_crosshaircolor_b", 0.0, 255.0);
    /// Color a channel in sRGB byte units.
    color_a: Channel = Channel(255) => ("cl_crosshaircolor_a", 0.0, 255.0);
    /// Border r channel in sRGB byte units.
    border_r: Channel = Channel(0) => ("cl_crosshairoutline_r", 0.0, 255.0);
    /// Border g channel in sRGB byte units.
    border_g: Channel = Channel(0) => ("cl_crosshairoutline_g", 0.0, 255.0);
    /// Border b channel in sRGB byte units.
    border_b: Channel = Channel(0) => ("cl_crosshairoutline_b", 0.0, 255.0);
    /// Border a channel in sRGB byte units.
    border_a: Channel = Channel(255) => ("cl_crosshairoutline_a", 0.0, 255.0);
    /// Whether to draw the center dot.
    has_dot: bool = false => ("cl_crosshairdot", 0.0, 1.0);
    /// Whether to hide the upper bar.
    is_t_shape: bool = false => ("cl_crosshair_t", 0.0, 1.0);
    /// Whether to apply the supplied recoil displacement.
    is_recoil_enabled: bool = true => ("cl_crosshair_recoil", 0.0, 1.0);
    /// Additional permitted spread beyond 128 authored pixels.
    spread_limit: f32 = 255.0 => ("cl_crosshair_dynamic_spread_limit", 0.0, 255.0);
    /// Legacy split threshold in authored pixels.
    split_distance: f32 = 3.0 => ("cl_crosshair_dynamic_splitdist", 0.0, 127.0);
    /// Fraction of legacy bar length assigned to outer bars.
    split_ratio: f32 = 1.0 => ("cl_crosshair_dynamic_maxdist_splitratio", 0.0, 1.0);
    /// Inner split opacity multiplier.
    inner_alpha: f32 = 0.0 => ("cl_crosshair_dynamic_splitalpha_innermod", 0.0, 1.0);
    /// Outer split opacity multiplier.
    outer_alpha: f32 = 1.0 => ("cl_crosshair_dynamic_splitalpha_outermod", 0.3, 1.0);
    /// Whether to draw the friendly-aim warning.
    is_friendly: bool = true => ("cl_crosshair_friendly_warning", 0.0, 1.0);
    /// WeaponState line thickness in physical pixels; menu range.
    sniper_width: f32 = 1.0 => ("cl_crosshair_sniper_width", 1.0, 6.0);
    /// Whether the optical sight uses the configured crosshair color.
    uses_ironsight_color: bool = false => ("cl_ironsight_usecrosshaircolor", 0.0, 1.0);
    /// Dimensionless optical-sight dot scale.
    ironsight_scale: f32 = 1.0 => ("cl_ironsight_dot_scale", 0.1, 2.0);
    /// Whether scope geometry shows supplied inaccuracy.
    is_sniper_inaccuracy_enabled: bool = true => ("cl_sniper_show_inaccuracy", 0.0, 1.0);
    /// Whether the scope remains visible during the cycling phase.
    is_unscope_delayed: bool = false => ("cl_sniper_delay_unscope", 0.0, 1.0);
    /// Whether the scope is visible when the cycling phase finishes.
    has_auto_rezoom: bool = true => ("cl_sniper_auto_rezoom", 0.0, 1.0);
    /// Whether the grenade guide retains the ordinary crosshair.
    is_crosshair_kept: bool = true => ("cl_grenadecrosshair_keepusercrosshair", 0.0, 1.0);
    /// Which observed players can supply the displayed crosshair.
    observer: ObserverPolicy = ObserverPolicy::Everyone => ("cl_show_observer_crosshair", 0.0, 2.0);
    /// Which configuration is displayed for an observed bot.
    bot: BotPolicy = BotPolicy::Default => ("cl_observed_bot_crosshair", 0.0, 2.0);
    /// Whether the flash grenade guide is enabled.
    is_flash_enabled: bool = true => ("cl_grenadecrosshair_flash", 0.0, 1.0);
    /// Delay before displaying the flash grenade guide.
    flash_delay: Delay = Delay(Duration::from_secs(2)) => ("cl_grenadecrosshairdelay_flash", 0.0, 5.0);
    /// Whether the explosive grenade guide is enabled.
    is_explosive_enabled: bool = true => ("cl_grenadecrosshair_explosive", 0.0, 1.0);
    /// Delay before displaying the explosive grenade guide.
    explosive_delay: Delay = Delay(Duration::from_secs(2)) => ("cl_grenadecrosshairdelay_explosive", 0.0, 5.0);
    /// Whether the fire grenade guide is enabled.
    is_fire_enabled: bool = true => ("cl_grenadecrosshair_fire", 0.0, 1.0);
    /// Delay before displaying the fire grenade guide.
    fire_delay: Delay = Delay(Duration::from_secs(2)) => ("cl_grenadecrosshairdelay_fire", 0.0, 5.0);
    /// Whether the smoke grenade guide is enabled.
    is_smoke_enabled: bool = true => ("cl_grenadecrosshair_smoke", 0.0, 1.0);
    /// Delay before displaying the smoke grenade guide.
    smoke_delay: Delay = Delay(Duration::from_secs(2)) => ("cl_grenadecrosshairdelay_smoke", 0.0, 5.0);
    /// Whether the decoy grenade guide is enabled.
    is_decoy_enabled: bool = true => ("cl_grenadecrosshair_decoy", 0.0, 1.0);
    /// Delay before displaying the decoy grenade guide.
    decoy_delay: Delay = Delay(Duration::from_secs(2)) => ("cl_grenadecrosshairdelay_decoy", 0.0, 5.0);
}

impl Settings {
    /// Resolves byte color channels to the renderer's linear color space.
    pub(super) fn color(&self) -> LinearRgba {
        Srgba::rgba_u8(
            self.color_r.0,
            self.color_g.0,
            self.color_b.0,
            self.color_a.0,
        )
        .into()
    }

    /// Resolves the independently configured outline color.
    pub(super) fn border(&self) -> LinearRgba {
        Srgba::rgba_u8(
            self.border_r.0,
            self.border_g.0,
            self.border_b.0,
            self.border_a.0,
        )
        .into()
    }

    /// Resolves each grenade's independently configured visibility and delay.
    pub(super) fn grenade_delay(&self, kind: crate::Grenade) -> Option<Duration> {
        use crate::Grenade;
        let (is_enabled, delay) = match kind {
            Grenade::Flash => (self.is_flash_enabled, self.flash_delay),
            Grenade::Explosive => (self.is_explosive_enabled, self.explosive_delay),
            Grenade::Fire => (self.is_fire_enabled, self.fire_delay),
            Grenade::Smoke => (self.is_smoke_enabled, self.smoke_delay),
            Grenade::Decoy => (self.is_decoy_enabled, self.decoy_delay),
        };
        is_enabled.then_some(delay.0)
    }
}
