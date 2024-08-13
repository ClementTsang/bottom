use tui::widgets::Borders;

// Default widget ID
pub const DEFAULT_WIDGET_ID: u64 = 56709;

// How much data is SHOWN
pub const DEFAULT_TIME_MILLISECONDS: u64 = 60 * 1000; // Defaults to 1 min.
pub const STALE_MIN_MILLISECONDS: u64 = 30 * 1000; // Lowest is 30 seconds
pub const TIME_CHANGE_MILLISECONDS: u64 = 15 * 1000; // How much to increment each time
pub const AUTOHIDE_TIMEOUT_MILLISECONDS: u64 = 5000; // 5 seconds to autohide

pub const TICK_RATE_IN_MILLISECONDS: u64 = 200;
// How fast the screen refreshes
pub const DEFAULT_REFRESH_RATE_IN_MILLISECONDS: u64 = 1000;
pub const MAX_KEY_TIMEOUT_IN_MILLISECONDS: u64 = 1000;

// Limits for when we should stop showing table gaps/labels (anything less means
// not shown)
pub const TABLE_GAP_HEIGHT_LIMIT: u16 = 7;
pub const TIME_LABEL_HEIGHT_LIMIT: u16 = 7;

// Side borders
pub const SIDE_BORDERS: Borders = Borders::LEFT.union(Borders::RIGHT);

// Help text
const HELP_CONTENTS_TEXT: [&str; 10] = [
    "Either scroll or press the number key to go to the corresponding help menu section:",
    "1 - General",
    "2 - CPU widget",
    "3 - Process widget",
    "4 - Process search widget",
    "5 - Process sort widget",
    "6 - Temperature widget",
    "7 - Disk widget",
    "8 - Battery widget",
    "9 - Basic memory widget",
];

// TODO [Help]: Search in help?
// TODO [Help]: Move to using tables for easier formatting?
pub(crate) const GENERAL_HELP_TEXT: [&str; 32] = [
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
    "PgUp, PgDown     Scroll up/down a table by a page",
    "Ctrl-u, Ctrl-d   Scroll up/down a table by half a page",
    "Mouse scroll     Scroll through the tables or zoom in/out of charts by scrolling up/down",
    "Mouse click      Selects the clicked widget, table entry, dialog option, or tab",
];

const CPU_HELP_TEXT: [&str; 2] = [
    "2 - CPU widget",
    "Mouse scroll     Scrolling over an CPU core/average shows only that entry on the chart",
];

const PROCESS_HELP_TEXT: [&str; 17] = [
    "3 - Process widget",
    "dd, F9           Kill the selected process",
    "c                Sort by CPU usage, press again to reverse",
    "m                Sort by memory usage, press again to reverse",
    "p                Sort by PID name, press again to reverse",
    "n                Sort by process name, press again to reverse",
    "Tab              Group/un-group processes with the same name",
    "Ctrl-f, /        Open process search widget",
    "P                Toggle between showing the full command or just the process name",
    "s, F6            Open process sort widget",
    "I                Invert current sort",
    "%                Toggle between values and percentages for memory usage",
    "t, F5            Toggle tree mode",
    "+, -, click      Collapse/expand a branch while in tree mode",
    "click on header  Sorts the entries by that column, click again to invert the sort",
    "C                Sort by GPU usage, press again to reverse",
    "M                Sort by GPU memory usage, press again to reverse",
];

const SEARCH_HELP_TEXT: [&str; 51] = [
    "4 - Process search widget",
    "Esc              Close the search widget (retains the filter)",
    "Ctrl-a           Skip to the start of the search query",
    "Ctrl-e           Skip to the end of the search query",
    "Ctrl-u           Clear the current search query",
    "Ctrl-w           Delete a word behind the cursor",
    "Ctrl-h           Delete the character behind the cursor",
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
    "read, r/s, rps   ex: read >= 1 b",
    "write, w/s, wps  ex: write <= 1 tb",
    "tread, t.read    ex: tread = 1",
    "twrite, t.write  ex: twrite = 1",
    "user             ex: user = root",
    "state            ex: state = running",
    "gpu%             ex: gpu% < 4.2",
    "gmem             ex: gmem < 100 kb",
    "gmem%            ex: gmem% < 4.2",
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

const SORT_HELP_TEXT: [&str; 6] = [
    "5 - Sort widget",
    "Down, 'j'        Scroll down in list",
    "Up, 'k'          Scroll up in list",
    "Mouse scroll     Scroll through sort widget",
    "Esc              Close the sort widget",
    "Enter            Sort by current selected column",
];

const TEMP_HELP_WIDGET: [&str; 3] = [
    "6 - Temperature widget",
    "'s'              Sort by sensor name, press again to reverse",
    "'t'              Sort by temperature, press again to reverse",
];

const DISK_HELP_WIDGET: [&str; 9] = [
    "7 - Disk widget",
    "'d'              Sort by disk name, press again to reverse",
    "'m'              Sort by disk mount, press again to reverse",
    "'u'              Sort by disk usage, press again to reverse",
    "'n'              Sort by disk free space, press again to reverse",
    "'t'              Sort by total disk space, press again to reverse",
    "'p'              Sort by disk usage percentage, press again to reverse",
    "'r'              Sort by disk read activity, press again to reverse",
    "'w'              Sort by disk write activity, press again to reverse",
];

const BATTERY_HELP_TEXT: [&str; 3] = [
    "8 - Battery widget",
    "Left             Go to previous battery",
    "Right            Go to next battery",
];

const BASIC_MEM_HELP_TEXT: [&str; 2] = [
    "9 - Basic memory widget",
    "%                Toggle between values and percentages for memory usage",
];

pub(crate) const HELP_TEXT: [&[&str]; HELP_CONTENTS_TEXT.len()] = [
    &HELP_CONTENTS_TEXT,
    &GENERAL_HELP_TEXT,
    &CPU_HELP_TEXT,
    &PROCESS_HELP_TEXT,
    &SEARCH_HELP_TEXT,
    &SORT_HELP_TEXT,
    &TEMP_HELP_WIDGET,
    &DISK_HELP_WIDGET,
    &BATTERY_HELP_TEXT,
    &BASIC_MEM_HELP_TEXT,
];

pub(crate) const DEFAULT_LAYOUT: &str = r#"
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
"#;

pub(crate) const DEFAULT_BATTERY_LAYOUT: &str = r#"
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
"#;

// TODO: Eventually deprecate this, or grab from a file.
pub(crate) const CONFIG_TEXT: &str = r#"# This is a default config file for bottom. All of the settings are commented
# out by default; if you wish to change them uncomment and modify as you see
# fit.

# This group of options represents a command-line option. Flags explicitly
# added when running (ie: btm -a) will override this config file if an option
# is also set here.
[flags]
# Whether to hide the average cpu entry.
#hide_avg_cpu = false
# Whether to use dot markers rather than braille.
#dot_marker = false
# The update rate of the application.
#rate = "1s"
# Whether to put the CPU legend to the left.
#cpu_left_legend = false
# Whether to set CPU% on a process to be based on the total CPU or just current usage.
#current_usage = false
# Whether to set CPU% on a process to be based on the total CPU or per-core CPU% (not divided by the number of cpus).
#unnormalized_cpu = false
# Whether to group processes with the same name together by default.
#group_processes = false
# Whether to make process searching case sensitive by default.
#case_sensitive = false
# Whether to make process searching look for matching the entire word by default.
#whole_word = false
# Whether to make process searching use regex by default.
#regex = false
# The temperature unit. One of the following, defaults to "c" for Celsius:
#temperature_type = "c"
##temperature_type = "k"
##temperature_type = "f"
##temperature_type = "kelvin"
##temperature_type = "fahrenheit"
##temperature_type = "celsius"
# The default time interval (in milliseconds).
#default_time_value = "60s"
# The time delta on each zoom in/out action (in milliseconds).
#time_delta = 15000
# Hides the time scale.
#hide_time = false
# Override layout default widget
#default_widget_type = "proc"
#default_widget_count = 1
# Expand selected widget upon starting the app
#expanded = true
# Use basic mode
#basic = false
# Use the old network legend style
#use_old_network_legend = false
# Remove space in tables
#hide_table_gap = false
# Show the battery widgets
#battery = false
# Disable mouse clicks
#disable_click = false
# Show memory values in the processes widget as values by default
#process_memory_as_value = false
# Show tree mode by default in the processes widget.
#tree = false
# Shows an indicator in table widgets tracking where in the list you are.
#show_table_scroll_position = false
# Show processes as their commands by default in the process widget.
#process_command = false
# Displays the network widget with binary prefixes.
#network_use_binary_prefix = false
# Displays the network widget using bytes.
#network_use_bytes = false
# Displays the network widget with a log scale.
#network_use_log = false
# Hides advanced options to stop a process on Unix-like systems.
#disable_advanced_kill = false
# Hide GPU(s) information
#disable_gpu = false
# Shows cache and buffer memory
#enable_cache_memory = false
# How much data is stored at once in terms of time.
#retention = "10m"
# Where to place the legend for the memory widget. One of "none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right".
#memory_legend = "TopRight"
# Where to place the legend for the network widget. One of "none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right".
#network_legend = "TopRight"

# Processes widget configuration
#[processes]
# The columns shown by the process widget. The following columns are supported:
# PID, Name, CPU%, Mem%, R/s, W/s, T.Read, T.Write, User, State, Time, GMem%, GPU%
#columns = ["PID", "Name", "CPU%", "Mem%", "R/s", "W/s", "T.Read", "T.Write", "User", "State", "GMem%", "GPU%"]

# CPU widget configuration
#[cpu]
# One of "all" (default), "average"/"avg"
# default = "average"

# Disk widget configuration
#[disk]
#[disk.name_filter]
#is_list_ignored = true
#list = ["/dev/sda\\d+", "/dev/nvme0n1p2"]
#regex = true
#case_sensitive = false
#whole_word = false

#[disk.mount_filter]
#is_list_ignored = true
#list = ["/mnt/.*", "/boot"]
#regex = true
#case_sensitive = false
#whole_word = false

# Temperature widget configuration
#[temperature]
#[temperature.sensor_filter]
#is_list_ignored = true
#list = ["cpu", "wifi"]
#regex = false
#case_sensitive = false
#whole_word = false

# Network widget configuration
#[network]
#[network.interface_filter]
#is_list_ignored = true
#list = ["virbr0.*"]
#regex = true
#case_sensitive = false
#whole_word = false

# These are all the components that support custom theming.  Note that colour support
# will depend on terminal support.
#[styles] # Uncomment if you want to use custom styling

# Built-in themes. Valid values are:
# - "default"
# - "default-light"
# - "gruvbox"
# - "gruvbox-light"
# - "nord"
# - "nord-light".
#
# This will have the lowest precedence if a custom colour palette is set,
# or overridden if the command-line flag for a built-in theme is set.
#theme = "default"

#[styles.cpu]
#all_entry_color = "green"
#avg_entry_color = "red"
#cpu_core_colors = ["light magenta", "light yellow", "light cyan", "light green", "light blue", "cyan", "green", "blue"]

#[styles.memory]
#ram_color = "light magenta"
#cache_color = "light red"
#swap_color = "light yellow"
#arc_color = "light cyan"
#gpu_colors = ["light blue", "light red", "cyan", "green", "blue", "red"]

#[styles.network]
#rx_color = "light magenta"
#tx_color = "light yellow"
#rx_total_color = "light cyan"
#tx_total_color = "light green"

#[styles.battery]
#high_battery_color = "green"
#medium_battery_color = "yellow"
#low_battery_color = "red"

#[styles.tables]
#headers = {color = "light blue", bold = true}

#[styles.graphs]
#graph_color = "gray"
#legend_text = {color = "gray"}

#[styles.widgets]
#border_color = "gray"
#selected_border_color = "light blue"
#widget_title = {color = "gray"}
#text = {color = "gray"}
#selected_text = {color = "black", bg_color = "light blue"}
#disabled_text = {color = "dark gray"}

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
"#;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn help_menu_matches_entry_len() {
        // The two match since HELP_TEXT contains HELP_CONTENTS_TEXT as an entry
        assert_eq!(
            HELP_CONTENTS_TEXT.len(),
            HELP_TEXT.len(),
            "the two should be equal, or this test should be updated"
        )
    }

    #[test]
    fn help_menu_text_has_sections() {
        for (itx, line) in HELP_TEXT.iter().enumerate() {
            if itx > 0 {
                assert!(line.len() >= 2, "each section should be at least 2 lines");
                assert!(line[0].contains(" - "), "each section should have a header");
            }
        }
    }

    /// This test exists because previously, [`SIDE_BORDERS`] was set
    /// incorrectly after I moved from tui-rs to ratatui.
    #[test]
    fn assert_side_border_bits_match() {
        assert_eq!(
            SIDE_BORDERS,
            Borders::ALL.difference(Borders::TOP.union(Borders::BOTTOM))
        )
    }

    /// Checks that the default config is valid.
    #[test]
    #[cfg(feature = "default")]
    fn check_default_config() {
        use regex::Regex;

        use crate::options::Config;

        let default_config = Regex::new(r"(?m)^#([a-zA-Z\[])")
            .unwrap()
            .replace_all(CONFIG_TEXT, "$1");

        let default_config = Regex::new(r"(?m)^#(\s\s+)([a-zA-Z\[])")
            .unwrap()
            .replace_all(&default_config, "$2");

        let _config: Config =
            toml_edit::de::from_str(&default_config).expect("can parse default config");

        // TODO: Check this.
        // assert_eq!(config, Config::default());
    }
}
