#[cfg(feature = "derive")]
pub use bevy_spliff_derive as derive;

pub mod core;
pub mod joins;

pub mod prelude {
    pub use super::core::*;
    #[cfg(feature = "derive")]
    pub use super::derive::*;
    pub use super::joins::prelude::*;
}
