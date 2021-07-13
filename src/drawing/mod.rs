mod rendering;
pub use rendering::*;

mod widgets;
use tui::{backend::Backend, Terminal};
pub use widgets::*;

pub mod event;
pub use event::*;

pub mod hasher;
pub use hasher::Hasher;

use crate::utils::error;

/// The paint function.  Draws the entire app.
/// Intended to go through a few phases:
///
/// 0. Create our "Element" tree from "Widgets".  This is from our given [`Widget`] representation.
///    This should have access to app state, and we can do caching checks here to see if we can avoid computations.
/// 1. Set up our "layout", an intermediate representation of our actual layout before we draw it.  We are armed with the widths
///    each widget wants.  We also want to aggressively cache to avoid any computations that depend on layouts.
/// 2. Draw using each node's draw function.
/// 3. Cache our current results for the next loop.
///
/// This is *sort of* like how Flutter does it.  We have our Widget tree, which is the non-instantiated representation
/// of our hierarchy.  We then actually instantiate this into Elements.  Then, we finally actually lay it out, which
/// would correspond to the RenderObject tree.  Then, we draw!
pub fn paint<B: Backend>(
    terminal: &mut Terminal<B>, root: &mut Element<'_, B>,
) -> error::Result<()> {
    terminal.draw(|ctx| {
        // Current implementation does zero caching.  Just blind draws.  Sorry~

        let layout = root.layout(ctx.size());
        root.draw(ctx, &layout);
    })?;

    Ok(())
}
