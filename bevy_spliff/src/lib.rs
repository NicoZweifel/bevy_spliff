#[cfg(feature = "derive")]
pub use bevy_spliff_derive as derive;

pub mod core;
pub mod joined;
pub mod behavior;

pub mod prelude {
    pub use super::core::prelude::*;
    #[cfg(feature = "derive")]
    pub use super::derive::*;
    pub use super::joined::prelude::*;
    pub use super::behavior::*;
}
