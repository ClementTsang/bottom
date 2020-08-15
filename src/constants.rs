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
pub const NUM_COLOURS: usize = 256;

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
pub const HELP_CONTENTS_TEXT: [&str; 7] = [
    "Press the corresponding numbers to jump to the section, or scroll:",
    "1 - General",
    "2 - CPU widget",
    "3 - Process widget",
    "4 - Process search widget",
    "5 - Process sort widget",
    "6 - Battery widget",
];

pub const GENERAL_HELP_TEXT: [&str; 29] = [
    "1 - General",
    "q, Ctrl-c      Quit",
    "Esc            Close dialog windows, search, widgets, or exit expanded mode",
    "Ctrl-r         Reset display and any collected data",
    "f              Freeze/unfreeze updating with new data",
    "Ctrl-Left,     ",
    "Shift-Left,    Move widget selection left",
    "H, A           ",
    "Ctrl-Right,    ",
    "Shift-Right,   Move widget selection right",
    "L, D           ",
    "Ctrl-Up,       ",
    "Shift-Up,      Move widget selection up",
    "K, W           ",
    "Ctrl-Down,     ",
    "Shift-Down,    Move widget selection down",
    "J, S           ",
    "Left, h        Move left within widget",
    "Down, j        Move down within widget",
    "Up, k          Move up within widget",
    "Right, l       Move right within widget",
    "?              Open help menu",
    "gg             Jump to the first entry",
    "G              Jump to the last entry",
    "e              Expand the currently selected widget",
    "+              Zoom in on chart (decrease time range)",
    "-              Zoom out on chart (increase time range)",
    "=              Reset zoom",
    "Mouse scroll   Scroll through the tables or zoom in/out of charts by scrolling up/down",
];

pub const CPU_HELP_TEXT: [&str; 2] = [
    "2 - CPU widget",
    "Mouse scroll   Scrolling over an CPU core/average shows only that entry on the chart",
];

// TODO [Help]: Search in help?
// TODO [Help]: Move to using tables for easier formatting?
pub const PROCESS_HELP_TEXT: [&str; 11] = [
    "3 - Process widget",
    "dd             Kill the selected process",
    "c              Sort by CPU usage, press again to reverse sorting order",
    "m              Sort by memory usage, press again to reverse sorting order",
    "p              Sort by PID name, press again to reverse sorting order",
    "n              Sort by process name, press again to reverse sorting order",
    "Tab            Group/un-group processes with the same name",
    "Ctrl-f, /      Open process search widget",
    "P              Toggle between showing the full path or just the process name",
    "s, F6          Open process sort widget",
    "I              Invert current sort",
];

pub const SEARCH_HELP_TEXT: [&str; 43] = [
    "4 - Process search widget",
    "Tab            Toggle between searching for PID and name",
    "Esc            Close the search widget (retains the filter)",
    "Ctrl-a         Skip to the start of the search query",
    "Ctrl-e         Skip to the end of the search query",
    "Ctrl-u         Clear the current search query",
    "Backspace      Delete the character behind the cursor",
    "Delete         Delete the character at the cursor",
    "Alt-c/F1       Toggle matching case",
    "Alt-w/F2       Toggle matching the entire word",
    "Alt-r/F3       Toggle using regex",
    "Left, Alt-h    Move cursor left",
    "Right, Alt-l   Move cursor right",
    "",
    "Search keywords:",
    "pid            ex: pid 825",
    "cpu            ex: cpu > 4.2",
    "mem            ex: mem < 4.2",
    "read           ex: read >= 1 b",
    "write          ex: write <= 1 tb",
    "tread          ex: tread = 1",
    "twrite         ex: twrite = 1",
    "",
    "Comparison operators:",
    "=              ex: cpu = 1",
    ">              ex: cpu > 1",
    "<              ex: cpu < 1",
    ">=             ex: cpu >= 1",
    "<=             ex: cpu <= 1",
    "",
    "Logical operators:",
    "and/&&/<Space> ex: btm and cpu > 1 and mem > 1",
    "or/||          ex: btm or firefox",
    "",
    "Supported units:",
    "B              ex: read > 1 b",
    "KB             ex: read > 1 kb",
    "MB             ex: read > 1 mb",
    "TB             ex: read > 1 tb",
    "KiB            ex: read > 1 kib",
    "MiB            ex: read > 1 mib",
    "GiB            ex: read > 1 gib",
    "TiB            ex: read > 1 tib",
];

pub const SORT_HELP_TEXT: [&str; 6] = [
    "5 - Sort widget",
    "Down, 'j'      Scroll down in list",
    "Up, 'k'        Scroll up in list",
    "Mouse scroll   Scroll through sort widget",
    "Esc            Close the sort widget",
    "Enter          Sort by current selected column",
];

pub const BATTERY_HELP_TEXT: [&str; 3] = [
    "6 - Battery widget",
    "Left           Go to previous battery",
    "Right          Go to next battery",
];

lazy_static! {
    pub static ref HELP_TEXT: Vec<Vec<&'static str>> = vec![
        HELP_CONTENTS_TEXT.to_vec(),
        GENERAL_HELP_TEXT.to_vec(),
        CPU_HELP_TEXT.to_vec(),
        PROCESS_HELP_TEXT.to_vec(),
        SEARCH_HELP_TEXT.to_vec(),
        SORT_HELP_TEXT.to_vec(),
        BATTERY_HELP_TEXT.to_vec(),
    ];
}

// Default layout
pub const DEFAULT_LAYOUT: &str = r##"
[[row]]
  ratio=30
  [[row.child]]
  type="cpu"
[[row]]
    ratio=40
    [[row.child]]
      ratio=4
      type="mem"
    [[row.child]]
      ratio=3
      [[row.child.child]]
        type="temp"
      [[row.child.child]]
        type="disk"
[[row]]
  ratio=30
  [[row.child]]
    type="net"
  [[row.child]]
    type="proc"
    default=true
"##;

// Config and flags
pub const DEFAULT_CONFIG_FILE_PATH: &str = "bottom/bottom.toml";

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
