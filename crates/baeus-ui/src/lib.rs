#![recursion_limit = "4096"]

pub mod components;
pub mod icons;
pub mod layout;
pub mod models;
pub mod theme;
pub mod views;

/// Re-export GPUI prelude for convenient access by consumers.
pub use gpui;
pub use gpui_component;
