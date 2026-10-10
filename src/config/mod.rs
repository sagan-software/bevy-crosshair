//! Closed configuration vocabularies and parsing errors shared by rendering inputs.

mod error;
mod style;
mod weapon;

pub use self::error::SettingError;
pub use self::style::Style;
pub use self::weapon::Grenade;
pub use self::weapon::WeaponState;
