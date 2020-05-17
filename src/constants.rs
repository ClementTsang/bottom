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

// Limits for when we should stop showing table gaps/labels (anything less means not shown)
pub const TABLE_GAP_HEIGHT_LIMIT: u16 = 7;
pub const TIME_LABEL_HEIGHT_LIMIT: u16 = 7;

// Side borders
lazy_static! {
    pub static ref SIDE_BORDERS: tui::widgets::Borders =
        tui::widgets::Borders::from_bits_truncate(20);
    pub static ref TOP_LEFT_RIGHT: tui::widgets::Borders =
        tui::widgets::Borders::from_bits_truncate(22);
    pub static ref BOTTOM_LEFT_RIGHT: tui::widgets::Borders =
        tui::widgets::Borders::from_bits_truncate(28);
    pub static ref DEFAULT_TEXT_STYLE: tui::style::Style =
        tui::style::Style::default().fg(tui::style::Color::Gray);
    pub static ref DEFAULT_HEADER_STYLE: tui::style::Style =
        tui::style::Style::default().fg(tui::style::Color::LightBlue);
}

// Help text
pub const HELP_CONTENTS_TEXT: [&str; 6] = [
    "Press the corresponding numbers to jump to the section, or scroll:\n",
    "1 - General\n",
    "2 - CPU widget\n",
    "3 - Process widget\n",
    "4 - Process search widget\n",
    "5 - Battery widget",
];

pub const GENERAL_HELP_TEXT: [&str; 20] = [
    "1 - General\n",
    "q, Ctrl-c      Quit\n",
    "Esc            Close dialog windows, search, widgets, or exit expanded mode\n",
    "Ctrl-r         Reset display and any collected data\n",
    "f              Freeze/unfreeze updating with new data\n",
    "Ctrl-Arrow     \n",
    "Shift-Arrow    Move to a different widget\n",
    "H/J/K/L        \n",
    "Left, h        Move left within widget\n",
    "Down, j        Move down within widget\n",
    "Up, k          Move up within widget\n",
    "Right, l       Move right within widget\n",
    "?              Open help menu\n",
    "gg             Jump to the first entry\n",
    "G              Jump to the last entry\n",
    "e              Expand the currently selected widget\n",
    "+              Zoom in on chart (decrease time range)\n",
    "-              Zoom out on chart (increase time range)\n",
    "=              Reset zoom\n",
    "Mouse scroll   Scroll through the tables or zoom in/out of charts by scrolling up/down",
];

pub const CPU_HELP_TEXT: [&str; 2] = [
    "2 - CPU widget\n",
    "Mouse scroll   Scrolling over an CPU core/average shows only that entry on the chart",
];

pub const PROCESS_HELP_TEXT: [&str; 8] = [
    "3 - Process widget\n",
    "dd             Kill the selected process\n",
    "c              Sort by CPU usage, press again to reverse sorting order\n",
    "m              Sort by memory usage, press again to reverse sorting order\n",
    "p              Sort by PID name, press again to reverse sorting order\n",
    "n              Sort by process name, press again to reverse sorting order\n",
    "Tab            Group/un-group processes with the same name\n",
    "Ctrl-f, /      Open process search widget",
];

pub const SEARCH_HELP_TEXT: [&str; 43] = [
    "4 - Process search widget\n",
    "Tab            Toggle between searching for PID and name\n",
    "Esc            Close the search widget (retains the filter)\n",
    "Ctrl-a         Skip to the start of the search query\n",
    "Ctrl-e         Skip to the end of the search query\n",
    "Ctrl-u         Clear the current search query\n",
    "Backspace      Delete the character behind the cursor\n",
    "Delete         Delete the character at the cursor\n",
    "Alt-c/F1       Toggle matching case\n",
    "Alt-w/F2       Toggle matching the entire word\n",
    "Alt-r/F3       Toggle using regex\n",
    "Left, Alt-h    Move cursor left\n",
    "Right, Alt-l   Move cursor right\n",
    "\n",
    "Search keywords:\n",
    "pid            ex: pid 825\n",
    "cpu            ex: cpu > 4.2\n",
    "mem            ex: mem < 4.2\n",
    "read           ex: read >= 1 b\n",
    "write          ex: write <= 1 tb\n",
    "tread          ex: tread = 1\n",
    "twrite         ex: twrite = 1\n",
    "\n",
    "Comparison operators:\n",
    "=              ex: cpu = 1\n",
    ">              ex: cpu > 1\n",
    "<              ex: cpu < 1\n",
    ">=             ex: cpu >= 1\n",
    "<=             ex: cpu <= 1\n",
    "\n",
    "Logical operators:\n",
    "and/&&/<Space> ex: btm and cpu > 1 and mem > 1\n",
    "or/||          ex: btm or firefox\n",
    "\n",
    "Supported units:\n",
    "B              ex: read > 1 b\n",
    "KB             ex: read > 1 kb\n",
    "MB             ex: read > 1 mb\n",
    "TB             ex: read > 1 tb\n",
    "KiB            ex: read > 1 kib\n",
    "MiB            ex: read > 1 mib\n",
    "GiB            ex: read > 1 gib\n",
    "TiB            ex: read > 1 tib",
];

pub const BATTERY_HELP_TEXT: [&str; 3] = [
    "5 - Battery widget\n",
    "Left           Go to previous battery\n",
    "Right          Go to next battery",
];

lazy_static! {
    pub static ref HELP_TEXT: Vec<Vec<&'static str>> = vec![
        HELP_CONTENTS_TEXT.to_vec(),
        GENERAL_HELP_TEXT.to_vec(),
        CPU_HELP_TEXT.to_vec(),
        PROCESS_HELP_TEXT.to_vec(),
        SEARCH_HELP_TEXT.to_vec(),
        BATTERY_HELP_TEXT.to_vec(),
    ];
}

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

# Whether to hide the average cpu entry.
#hide_avg_cpu = false

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

# Use the old network legend style
#use_old_network_legend = false

# Remove space in tables
#hide_table_gap = false

##########################################################

# These are all the components that support custom theming.  Note that colour support
# will, at the end of the day, depend on terminal support - for example, the
# macOS default Terminal does NOT like custom colours and it will glitch out.
[colors]

# Represents the colour of table headers (processes, CPU, disks, temperature).
#table_header_color="LightBlue"

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
#highlighted_border_color="LightBlue"

# Represents the colour of most text.
#text_color="Gray"

# Represents the colour of text that is selected.
#selected_text_color="Black"

# Represents the background colour of text that is selected.
#selected_bg_color="LightBlue"

# Represents the colour of the lines and text of the graph.
#graph_color="Gray"

# Represents the colours of the battery based on charge
#battery_colors = ["red", "yellow", "yellow", "green", "green", "green"]

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
