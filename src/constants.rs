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

// Limits for when we should stop showing table gaps/labels (anything less means not shown)
pub const TABLE_GAP_HEIGHT_LIMIT: u16 = 7;
pub const TIME_LABEL_HEIGHT_LIMIT: u16 = 7;

// Side borders
// FIXME: [REFACTOR] Switch to once_cell over lazy_static.
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

// FIXME: [HELP] I wanna update this before release... it's missing mouse too.
// Help text
pub const HELP_CONTENTS_TEXT: [&str; 8] = [
    "Press the corresponding numbers to jump to the section, or scroll:",
    "1 - General",
    "2 - CPU widget",
    "3 - Process widget",
    "4 - Process search widget",
    "5 - Process sort widget",
    "6 - Battery widget",
    "7 - Basic memory widget",
];

pub const GENERAL_HELP_TEXT: [&str; 29] = [
    "1 - General",
    "q, Ctrl-c        Quit",
    "Esc              Close dialog windows, search, widgets, or exit expanded mode",
    "Ctrl-r           Reset display and any collected data",
    "f                Freeze/unfreeze updating with new data",
    "Ctrl-Left,       ",
    "Shift-Left,      Move widget selection left",
    "H, A             ",
    "Ctrl-Right,      ",
    "Shift-Right,     Move widget selection right",
    "L, D             ",
    "Ctrl-Up,         ",
    "Shift-Up,        Move widget selection up",
    "K, W             ",
    "Ctrl-Down,       ",
    "Shift-Down,      Move widget selection down",
    "J, S             ",
    "Left, h          Move left within widget",
    "Down, j          Move down within widget",
    "Up, k            Move up within widget",
    "Right, l         Move right within widget",
    "?                Open help menu",
    "gg               Jump to the first entry",
    "G                Jump to the last entry",
    "e                Toggle expanding the currently selected widget",
    "+                Zoom in on chart (decrease time range)",
    "-                Zoom out on chart (increase time range)",
    "=                Reset zoom",
    "Mouse scroll     Scroll through the tables or zoom in/out of charts by scrolling up/down",
];

pub const CPU_HELP_TEXT: [&str; 2] = [
    "2 - CPU widget\n",
    "Mouse scroll     Scrolling over an CPU core/average shows only that entry on the chart",
];

// TODO [Help]: Search in help?
// TODO [Help]: Move to using tables for easier formatting?
pub const PROCESS_HELP_TEXT: [&str; 13] = [
    "3 - Process widget",
    "dd               Kill the selected process",
    "c                Sort by CPU usage, press again to reverse sorting order",
    "m                Sort by memory usage, press again to reverse sorting order",
    "p                Sort by PID name, press again to reverse sorting order",
    "n                Sort by process name, press again to reverse sorting order",
    "Tab              Group/un-group processes with the same name",
    "Ctrl-f, /        Open process search widget",
    "P                Toggle between showing the full command or just the process name",
    "s, F6            Open process sort widget",
    "I                Invert current sort",
    "%                Toggle between values and percentages for memory usage",
    "t, F5            Toggle tree mode",
];

pub const SEARCH_HELP_TEXT: [&str; 46] = [
    "4 - Process search widget",
    "Tab              Toggle between searching for PID and name",
    "Esc              Close the search widget (retains the filter)",
    "Ctrl-a           Skip to the start of the search query",
    "Ctrl-e           Skip to the end of the search query",
    "Ctrl-u           Clear the current search query",
    "Backspace        Delete the character behind the cursor",
    "Delete           Delete the character at the cursor",
    "Alt-c, F1        Toggle matching case",
    "Alt-w, F2        Toggle matching the entire word",
    "Alt-r, F3        Toggle using regex",
    "Left, Alt-h      Move cursor left",
    "Right, Alt-l     Move cursor right",
    "",
    "Supported search types:",
    "<by name/cmd>    ex: btm",
    "pid              ex: pid 825",
    "cpu, cpu%        ex: cpu > 4.2",
    "mem, mem%        ex: mem < 4.2",
    "memb             ex: memb < 100 kb",
    "read, r/s        ex: read >= 1 b",
    "write, w/s       ex: write <= 1 tb",
    "tread, t.read    ex: tread = 1",
    "twrite, t.write  ex: twrite = 1",
    "state            ex: state = running",
    "",
    "Comparison operators:",
    "=                ex: cpu = 1",
    ">                ex: cpu > 1",
    "<                ex: cpu < 1",
    ">=               ex: cpu >= 1",
    "<=               ex: cpu <= 1",
    "",
    "Logical operators:",
    "and, &&, <Space> ex: btm and cpu > 1 and mem > 1",
    "or, ||           ex: btm or firefox",
    "",
    "Supported units:",
    "B                ex: read > 1 b",
    "KB               ex: read > 1 kb",
    "MB               ex: read > 1 mb",
    "TB               ex: read > 1 tb",
    "KiB              ex: read > 1 kib",
    "MiB              ex: read > 1 mib",
    "GiB              ex: read > 1 gib",
    "TiB              ex: read > 1 tib",
];

pub const SORT_HELP_TEXT: [&str; 6] = [
    "5 - Sort widget\n",
    "Down, 'j'        Scroll down in list",
    "Up, 'k'          Scroll up in list",
    "Mouse scroll     Scroll through sort widget",
    "Esc              Close the sort widget",
    "Enter            Sort by current selected column",
];

pub const BATTERY_HELP_TEXT: [&str; 3] = [
    "6 - Battery widget",
    "Left             Go to previous battery",
    "Right            Go to next battery",
];

pub const BASIC_MEM_HELP_TEXT: [&str; 2] = [
    "7 - Basic memory widget",
    "%                Toggle between values and percentages for memory usage",
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
        BASIC_MEM_HELP_TEXT.to_vec(),
    ];
}

// Default layouts
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

pub const DEFAULT_BATTERY_LAYOUT: &str = r##"
[[row]]
  ratio=30
  [[row.child]]
    ratio=2
  type="cpu"
  [[row.child]]
    ratio=1
  type="battery"
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

pub const CONFIG_TOP_HEAD: &str = r##"# This is bottom's config file.  Values in this config file will change when changed in the
# interface.  You can also manually change these values.  Be aware that contents of this file will be overwritten if something is
# changed in the application; you can disable writing via the --no_write flag or no_write config option.

"##;

pub const CONFIG_DISPLAY_OPTIONS_HEAD: &str = r##"
# These options represent settings that affect how bottom functions.
# If a setting here corresponds to command-line flag, then the flag will temporarily override
# the setting.
"##;

pub const CONFIG_COLOUR_HEAD: &str = r##"
# These options represent colour values for various parts of bottom.  Note that colour support
# will ultimately depend on the terminal - for example, the Terminal for macOS does NOT like
# custom colours and it may glitch out.
"##;

pub const CONFIG_LAYOUT_HEAD: &str = r##"
# These options represent how bottom will lay out its widgets.  Layouts follow a pattern like this:
# [[row]] represents a row in the application.
# [[row.child]] represents either a widget or a column.
# [[row.child.child]] represents a widget.
#
# All widgets must have the valid type value set to one of ["cpu", "mem", "proc", "net", "temp", "disk", "empty"].
# All layout components have a ratio value - if this is not set, then it defaults to 1. 
"##;

pub const CONFIG_FILTER_HEAD: &str = r##"
# These options represent disabled entries for the temperature and disk widgets.
"##;
