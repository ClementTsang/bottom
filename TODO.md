# To-Do List

Note this will probably migrate to GitHub's native Issues; this was mostly for personal use during early stages.

- Rebalance cpu usage in process by using current value (it's currently just summing to 100%)

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
