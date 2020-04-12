use lazy_static::lazy_static;

// Default widget ID
pub const DEFAULT_WIDGET_ID: u64 = 56709;

// How long to store data.
pub const STALE_MAX_MILLISECONDS: u64 = 600 * 1000; // Keep 10 minutes of data.

// How much data is SHOWN
pub const DEFAULT_TIME_MILLISECONDS: u64 = 60 * 1000; // Defaults to 1 min.
pub const STALE_MIN_MILLISECONDS: u64 = 30 * 1000; // Lowest is 30 seconds
pub const TIME_CHANGE_MILLISECONDS: u64 = 15 * 1000; // How much to increment each time
pub const AUTOHIDE_TIMEOUT_MILLISECONDS: u64 = 5000; // 5 seconds to autohide

pub const TICK_RATE_IN_MILLISECONDS: u64 = 200;
// How fast the screen refreshes
pub const DEFAULT_REFRESH_RATE_IN_MILLISECONDS: u64 = 1000;
pub const MAX_KEY_TIMEOUT_IN_MILLISECONDS: u64 = 1000;
// Number of colours to generate for the CPU chart/table
pub const NUM_COLOURS: i32 = 256;

// Canvas stuff
// The minimum threshold when resizing tables
pub const FORCE_MIN_THRESHOLD: usize = 5;

// Side borders
lazy_static! {
    pub static ref SIDE_BORDERS: tui::widgets::Borders =
        tui::widgets::Borders::from_bits_truncate(20);
    pub static ref DEFAULT_TEXT_STYLE: tui::style::Style =
        tui::style::Style::default().fg(tui::style::Color::Gray);
    pub static ref DEFAULT_HEADER_STYLE: tui::style::Style =
        tui::style::Style::default().fg(tui::style::Color::LightBlue);
    pub static ref INVALID_REGEX_STYLE: tui::style::Style =
        tui::style::Style::default().fg(tui::style::Color::Red);
}

// Help text
pub const GENERAL_HELP_TEXT: [&str; 18] = [
    "General Keybindings\n\n",
    "q, Ctrl-c      Quit bottom\n",
    "Esc            Close filters, dialog boxes, etc.\n",
    "Ctrl-r         Reset all data\n",
    "f              Freeze display\n",
    "Ctrl-Arrow     Change your selected widget\n",
    "Shift-Arrow    Change your selected widget\n",
    "H/J/K/L        Change your selected widget up/down/left/right\n",
    "Up, k          Move cursor up\n",
    "Down, j        Move cursor down\n",
    "?              Open the help screen\n",
    "gg             Skip to the first entry of a list\n",
    "G              Skip to the last entry of a list\n",
    "Enter          Maximize the currently selected widget\n",
    "/              Filter out graph lines (only CPU at the moment)\n",
    "+              Zoom in (decrease time range)\n",
    "-              Zoom out (increase time range)\n",
    "=              Reset zoom\n",
];

pub const PROCESS_HELP_TEXT: [&str; 8] = [
    "Process Keybindings\n\n",
    "dd, Delete     Kill the highlighted process\n",
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

// Config and flags
pub const DEFAULT_UNIX_CONFIG_FILE_PATH: &str = ".config/bottom/bottom.toml";
pub const DEFAULT_WINDOWS_CONFIG_FILE_PATH: &str = "bottom/bottom.toml";

// Default config file
pub const DEFAULT_CONFIG_CONTENT: &str = r##"
# This is a default config file for bottom.  All of the settings are commented
# out by default; if you wish to change them uncomment and modify as you see
# fit.

# This group of options represents a command-line flag/option.  Flags explicitly
# added when running (ie: btm -a) will override this config file if an option
# is also set here.
[flags]

# Whether to display an average cpu entry.
#avg_cpu = false

# Whether to use dot markers rather than braille.
#dot_marker = false

# The update rate of the application.
#rate = 1000

# Whether to put the CPU legend to the left.
#left_legend = false

# Whether to set CPU% on a process to be based on the total CPU or just current usage.
#current_usage = false

# Whether to group processes with the same name together by default.
#group_processes = false

# Whether to make process searching case sensitive by default.
#case_sensitive = false

# Whether to make process searching look for matching the entire word by default.
#whole_word = false

# Whether to make process searching use regex by default.
#regex = false

# Whether to show CPU entries in the legend when they are hidden.
#show_disabled_data = false

# Defaults to Celsius.  Temperature is one of:
#temperature_type = "k"
#temperature_type = "f"
#temperature_type = "c"
#temperature_type = "kelvin"
#temperature_type = "fahrenheit"
#temperature_type = "celsius"

# The default time interval (in milliseconds).
#default_time_value = 60000

# The time delta on each zoom in/out action (in milliseconds).
#time_delta = 15000

# Override layout default widget
#default_widget_type = "proc"
#default_widget_count = 1

# Use basic mode
#basic = false

##########################################################

# These are all the components that support custom theming.  Note that colour support
# will, at the end of the day, depend on terminal support - for example, the
# macOS default Terminal does NOT like custom colours and it will glitch out.
[colors]

# Represents the colour of table headers (processes, CPU, disks, temperature).
#table_header_color="Cyan"

# Represents the colour of the label each widget has.
#widget_title_color="Gray"

# Represents the average CPU color.
#avg_cpu_color="Red"

# Represents the colour the core will use in the CPU legend and graph.
#cpu_core_colors=["LightMagenta", "LightYellow", "LightCyan", "LightGreen", "LightBlue", "LightRed", "Cyan", "Green", "Blue", "Red"]

# Represents the colour RAM will use in the memory legend and graph.
#ram_color="LightMagenta"

# Represents the colour SWAP will use in the memory legend and graph.
#swap_color="LightYellow"

# Represents the colour rx will use in the network legend and graph.
#rx_color="LightCyan"

# Represents the colour tx will use in the network legend and graph.
#tx_color="LightGreen"

# Represents the colour of the border of unselected widgets.
#border_color="Gray"

# Represents the colour of the border of selected widgets.
#highlighted_border_color="Cyan"

# Represents the colour of most text.
#text_color="Gray"

# Represents the colour of text that is selected.
#selected_text_color="Black"

# Represents the background colour of text that is selected.
#selected_bg_color="Cyan"

# Represents the colour of the lines and text of the graph.
#graph_color="Gray"

##########################################################

# Layout - layouts follow a pattern like this:
# [[row]] represents a row in the application.
# [[row.child]] represents either a widget or a column.
# [[row.child.child]] represents a widget.
#
# All widgets must have the type value set to one of ["cpu", "mem", "proc", "net", "temp", "disk", "empty"].
# All layout components have a ratio value - if this is not set, then it defaults to 1. 

# The default widget layout:
#[[row]]
#  ratio=30
#  [[row.child]]
#  type="cpu"
#[[row]]
#    ratio=40
#    [[row.child]]
#      ratio=4
#      type="mem"
#    [[row.child]]
#      ratio=3
#      [[row.child.child]]
#        type="temp"
#      [[row.child.child]]
#        type="disk"
#[[row]]
#  ratio=30
#  [[row.child]]
#    type="net"
#  [[row.child]]
#    type="proc"
#    default=true
"##;
