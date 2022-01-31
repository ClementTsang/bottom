//! tuine is a wrapper around tui-rs that expands on it with state management and
//! event handling.
//!
//! tuine is inspired by a **ton** of other libraries and frameworks:
//!
//! - [Crochet](https://github.com/raphlinus/crochet)
//! - [Dioxus](https://github.com/DioxusLabs/dioxus)
//! - [Druid](https://github.com/linebender/druid)
//! - [Flutter](https://flutter.dev/)
//! - [Iced](https://github.com/iced-rs/iced)
//! - [Jetpack Compose](https://developer.android.com/jetpack/compose)
//! - [React](https://reactjs.org/)
//! - [Yew](https://yew.rs/)
//!
//! In addition, Raph Levien's post,
//! [*Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html),
//! was a fantastic source of information for someone like me who had basically zero knowledge heading in.
//!
//! None of this would be possible without these as reference points and sources of inspiration and learning!

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
