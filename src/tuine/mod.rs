//! tuine is a wrapper around tui-rs that expands on it with state management and
//! event handling.
//!
//! tuine is inspired by a **ton** of other libraries and frameworks, like:
//!
//! - [Crochet](https://github.com/raphlinus/crochet)
//! - [Druid](https://github.com/linebender/druid)
//! - [Flutter](https://flutter.dev/)
//! - [Iced](https://github.com/iced-rs/iced)
//! - [React](https://reactjs.org/)
//! - [Yew](https://yew.rs/)

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
