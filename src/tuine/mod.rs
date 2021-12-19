//! tuine is a wrapper around tui-rs that expands upon state management and
//! event handling.
//!
//! tuine is inspired by a **ton** of other libraries and frameworks, like:
//! - Iced
//! - [crochet](https://github.com/raphlinus/crochet)
//! - Flutter
//! - React
//! - Yew

mod tui_rs;

pub mod component;
pub use component::*;

pub mod event;
pub use event::*;

pub mod application;
pub use application::*;

pub mod runtime;
pub use runtime::RuntimeEvent;

pub mod layout;
pub use layout::*;

pub mod element;
pub use element::*;

pub mod context;
pub use context::*;

pub mod screen;
pub use screen::*;

pub mod key;
pub use key::*;

pub mod state;
pub use state::*;
