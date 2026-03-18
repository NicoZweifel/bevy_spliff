#[cfg(feature = "derive")]
pub use bevy_spliff_derive as derive;

pub mod behavior;
pub mod core;
pub mod join;

pub mod prelude {
    pub use super::behavior::*;
    pub use super::core::prelude::*;
    #[cfg(feature = "derive")]
    pub use super::derive::*;
    pub use super::join::prelude::*;
}
