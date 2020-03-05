pub const STALE_MAX_MILLISECONDS: u128 = 60 * 1000;
// How long to store data.
pub const TIME_STARTS_FROM: u64 = 60 * 1000;
pub const TICK_RATE_IN_MILLISECONDS: u64 = 200;
// How fast the screen refreshes
pub const DEFAULT_REFRESH_RATE_IN_MILLISECONDS: u128 = 1000;
pub const MAX_KEY_TIMEOUT_IN_MILLISECONDS: u128 = 1000;
pub const NUM_COLOURS: i32 = 256;

// Config and flags
pub const DEFAULT_UNIX_CONFIG_FILE_PATH: &str = ".config/bottom/bottom.toml";
pub const DEFAULT_WINDOWS_CONFIG_FILE_PATH: &str = "bottom/bottom.toml";

// Help text
pub const GENERAL_HELP_TEXT: [&str; 15] = [
    "General Keybindings\n\n",
    "q, Ctrl-c      Quit bottom\n",
    "Esc            Close filters, dialog boxes, etc.\n",
    "Ctrl-r         Reset all data\n",
    "f              Freeze display\n",
    "Ctrl-Arrow     Move currently selected widget\n",
    "Shift-Arrow    Move currently selected widget\n",
    "H/J/K/L        Move currently selected widget up/down/left/right\n",
    "Up, k          Move cursor up\n",
    "Down, j        Move cursor down\n",
    "?              Open the help screen\n",
    "gg             Skip to the first entry of a list\n",
    "G              Skip to the last entry of a list\n",
    "Enter          Maximize the currently selected widget\n",
    "/              Filter out graph lines (only CPU at the moment)\n",
];

pub const PROCESS_HELP_TEXT: [&str; 8] = [
    "Process Keybindings\n\n",
    "dd             Kill the highlighted process\n",
    "c              Sort by CPU usage\n",
    "m              Sort by memory usage\n",
    "p              Sort by PID\n",
    "n              Sort by process name\n",
    "Tab            Group together processes with the same name\n",
    "Ctrl-f, /      Open up the search widget\n",
];

pub const SEARCH_HELP_TEXT: [&str; 13] = [
    "Search Keybindings\n\n",
    "Tab            Toggle between searching for PID and name.\n",
    "Esc            Close search widget\n",
    "Ctrl-a         Skip to the start of search widget\n",
    "Ctrl-e         Skip to the end of search widget\n",
    "Ctrl-u         Clear the current search query\n",
    "Backspace      Delete the character behind the cursor\n",
    "Delete         Delete the character at the cursor\n",
    "Left           Move cursor left\n",
    "Right          Move cursor right\n",
    "Alt-c/F1       Toggle whether to ignore case\n",
    "Alt-w/F2       Toggle whether to match the whole word\n",
    "Alt-r/F3       Toggle whether to use regex\n",
];

pub const DEFAULT_CONFIG_CONTENT: &str = r##"
# This is a default config file for bottom.  All of the settings are commented
# out by default; if you wish to change them uncomment and modify as you see
# fit.

# This group of options represents a command-line flag/option.  Flags explicitly
# added when running (ie: btm -a) will override this config file if an option
# is also set here.
[flags]

#avg_cpu = true
#dot_marker = false
#rate = 1000
#left_legend = false
#current_usage = false
#group_processes = false
#case_sensitive = false
#whole_word = true
#regex = true
#show_disabled_data = true

# Defaults to Celsius.  Temperature is one of:
#temperature_type = "k"
#temperature_type = "f"
#temperature_type = "c"
#temperature_type = "kelvin"
#temperature_type = "fahrenheit"
#temperature_type = "celsius"

# Defaults to processes.  Default widget is one of:
#default_widget = "cpu_default"
#default_widget = "memory_default"
#default_widget = "disk_default"
#default_widget = "temperature_default"
#default_widget = "network_default"
#default_widget = "process_default"


# These are all the components that support custom theming.  Currently, it only
# supports taking in a string representing a hex colour.  Note that colour support
# will, at the end of the day, depend on terminal support - for example, the
# macOS default Terminal does NOT like custom colours and it will glitch out.
#
# The default options here are based on gruvbox: https://github.com/morhetz/gruvbox
[colors]

# Represents the colour of table headers (processes, CPU, disks, temperature).
#table_header_color="#458588"

# Represents the colour of the label each widget has.
#widget_title_color="#cc241d"

# Represents the average CPU color
#avg_cpu_color="#d3869b"

# Represents the colour the core will use in the CPU legend and graph.
#cpu_core_colors=["#cc241d", "#98971a"]

# Represents the colour RAM will use in the memory legend and graph.
#ram_color="#fb4934"

# Represents the colour SWAP will use in the memory legend and graph.
#swap_color="#fabd2f"

# Represents the colour rx will use in the network legend and graph.
#rx_color="#458588"

# Represents the colour tx will use in the network legend and graph.
#tx_color="#689d6a"

# Represents the colour of the border of unselected widgets.
#border_color="#ebdbb2"

# Represents the colour of the border of selected widgets.
#highlighted_border_color="#fe8019"

# Represents the colour of most text.
#text_color="#ebdbb2"

# Represents the colour of text that is selected.
#selected_text_color="#282828"

# Represents the background colour of text that is selected.
#selected_bg_color="#458588"

# Represents the colour of the lines and text of the graph.
#graph_color="#ebdbb2"

# Represents the cursor's colour.
#cursor_color="#458588"
"##;
