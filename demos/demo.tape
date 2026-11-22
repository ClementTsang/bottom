# VHS for bottom.
# To use:
#   1. Install vhs, launch firefox and kitty. Install speedcrunch as that's our target application to kill.
#   2. Compile using `cargo build --release`
#   3. Run in the project directory `vhs demos/demo.tape`.
#   4. Wait for it to finish, this will take a few minutes.
#   5. Check the output gif in `demos/demo.gif`.
#   6. Optimize the gif size. What seems to work is resize to 960x540, then optimize. Tools like Ezgif are great, and aim for 2-3 MB.

# Output configuration
Output demos/demo.gif

# Terminal settings
Set Shell bash
Set FontSize 16
Set Width 1280
Set Height 720
Set Padding 10
Set Theme "Dracula"
Set Framerate 30
Set FontFamily "BlexMono Nerd Font"

Hide
Type "./target/release/btm -C sample_configs/demo_config.toml"
Enter

# Let it stabilize and show the initial view


# Done
Sleep 3s
Show
Sleep 3s
