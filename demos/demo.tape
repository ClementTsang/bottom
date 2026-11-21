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

Hide
Type "speedcrunch &"
Enter
Type "./target/release/btm -C sample_configs/demo_config.toml"
Enter

# Let it stabilize and show the initial view
Sleep 120s
Show

# Start. Have a sleep so there's at least a few seconds of stability.
Sleep 2s

# Expand
Type "e"
Sleep 1s

# Zoom out
Type@100ms "----------"
Sleep 1s

# Zoom back in
Type@100ms "++++++++++"
Sleep 1s

# Go to the CPU usage scroll
Ctrl+Right
Sleep 0.2s

# Scroll down
Down
Sleep 0.2s
Down
Sleep 0.2s
Down
Sleep 0.2s
Down
Sleep 0.2s
Down
Sleep 0.2s
Sleep 1s

# Scroll back to average
Up
Sleep 0.2s
Up
Sleep 0.2s
Up
Sleep 0.2s
Up
Sleep 0.2s
Sleep 1s

# Undo expand
Type "e"
Sleep 1s

# Now go to the memory widget
Sleep 1s
Ctrl+Left
Sleep 0.2s
Ctrl+Down
Sleep 0.2s
Sleep 0.5s

# Scroll out a few times
Type@100ms "---"
Sleep 1s

# Now go to the network widget
Sleep 1s
Ctrl+Down
Sleep 0.2s
Sleep 0.5s

# Scroll in twice
Type@50ms "++"
Sleep 1s

# Now go to the temperature widget and scroll around, then change sort
Sleep 1.5s
Ctrl+Up
Sleep 0.2s
Ctrl+Right
Sleep 0.2s
Sleep 1.1s
Down
Sleep 0.05s
Down
Sleep 0.05s
Down
Sleep 0.05s
Down
Sleep 0.05s
Down
Sleep 0.05s
Down
Sleep 0.05s
Down
Sleep 0.05s
Down
Sleep 0.05s
Sleep 1s
Type "gg"
Sleep 1s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "t"
Sleep 1.5s
Type "t"
Sleep 1.5s

# Go to disk and show off column sorting again
Sleep 1s
Ctrl+Down
Sleep 1s
Type "e"
Sleep 1s
Type "d"
Sleep 1.5s
Type "m"
Sleep 1.5s
Type "u"
Sleep 1.5s
Sleep 1s
Type "e"
Sleep 1s

# Go to processes
Sleep 1s
Ctrl+Down
Sleep 1s
Type "e"
Sleep 1s

# Show off process sorting
Type "p"
Sleep 1s
Type "p"
Sleep 1.5s
Type "c"
Sleep 1.5s
Type "P"
Sleep 2s
Type "P"

# Show off tree mode
Sleep 1s
Type "t"
Sleep 3s
Type "-"
Sleep 0.5s
Down
Type "-"
Sleep 0.5s
Type "-"
Sleep 0.5s
Up
Type "-"
Sleep 1s
Type "t"

# Show off search
Sleep 1s
Type "/"
Type@100ms "firefox OR kitty"
Sleep 2s
Ctrl+u
Sleep 1s
Alt+w
Sleep 0.5s
Type @250ms"pid=37 OR cpu>1"
Sleep 2s
Ctrl+u
Sleep 0.2s
Alt+w
Sleep 0.2s
Alt+r
Type@100ms "kworker/1[1-3]"
Sleep 2s
Ctrl+u

# Show off process killing
Sleep 1s
Type@100ms "speedcrunch"
Sleep 1s
Ctrl+Up
Delete
Sleep 1s
Enter
Escape
Sleep 0.5s
Escape
Sleep 0.5s
Type "/"
Sleep 0.5s
Ctrl+u
Sleep 0.5s
Escape

# Done
Sleep 3s
