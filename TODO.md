# To-Do List

Note this will probably migrate to GitHub's native Issues; this was mostly for personal use during early stages.

## Pre-release (bare minimum)

- ~~Get each function working as a POC~~

- ~~Separate each component for readability, finalize project structure~~

- ~~Refreshing - how are we doing that? Are we allowing individual refresh periods per component?~~

- ~~Write tui display, charting~~

- ~~FIX PROCESSES AHHHHHH~~

- ~~Scrolling in at least processes~~

- Keybindings - I want to do at least arrow keys and dd.

- ~~Legend gets in the way at too small of a height... maybe modify tui a bit more to fix this.~~

## After making public

- Scrolling support for temp/disk

- Travis

- Refactoring! Please.

- Scaling in and out (zoom), may need to show zoom levels

- More keybinds

- Tests

- Mouse + key events conflict? Make it so that some events don't clog up the loop if they are not valid keys!

- Header should be clear on current sorting direction!

- It would be maybe a good idea to see if we can run the process calculation across ALL cpus...? Might be more accurate.

- ~~Add custom error because it's really messy~~ Done, but need to implement across rest of app!

- Remove any `unwrap()`, ensure no crashing! Might have to use this: <https://doc.rust-lang.org/std/panic/fn.catch_unwind.html>

- Truncate columns if needed for tables

- Efficiency!!!

- Filtering in processes (that is, allow searching)

- Help screen

- Modularity

- Probably good to add a "are you sure" to dd-ing...
