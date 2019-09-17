# To-Do List

Note this will probably migrate to GitHub's native Issues; this was mostly for personal use during early stages.

## Pre-release (bare minimum)

* ~~Get each function working as a POC~~

* ~~Separate each component for readability, finalize project structure~~

* ~~Refreshing - how are we doing that?  Are we allowing individual refresh periods per component?~~

* ~~Write tui display, charting~~

* ~~FIX PROCESSES AHHHHHH~~

* ~~Scrolling in at least processes~~

* Keybindings - I want to do at least arrow keys and dd.

* ~~Legend gets in the way at too small of a height... maybe modify tui a bit more to fix this.~~

## After making public

* Travis

* Scaling in and out (zoom), may need to show zoom levels

* More keybinds

* Tests

* Mouse + key events conflict?  Make it so that some events don't clog up the loop if they are not valid keys!

* Header should be clear on current sorting direction!

* It would be maybe a good idea to see if we can run the process calculation across ALL cpus...?  Might be more accurate.

* ~~Add custom error because it's really messy~~ Done, but need to implement across rest of app!

* Remove any ``unwrap()``, ensure no crashing!  Might have to use this: <https://doc.rust-lang.org/std/panic/fn.catch_unwind.html>

* Scrolling event in lists

* Switching between panels

* Truncate columns if needed for tables

* Refactor everything because it's a mess

* Test for Windows support, mac support, other.  May be doable, depends on sysinfo and how much I know about other OSes probably.

* Efficiency!!!

* Filtering in processes (that is, allow searching)

* Help screen

* Modularity

* ~~Potentially process managing?  Depends on the libraries...~~ Done on Linux!

* Probably good to add a "are you sure" to dd-ing...
