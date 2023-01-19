use once_cell::sync::Lazy;

use crate::options::ConfigColours;

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

// Limits for when we should stop showing table gaps/labels (anything less means not shown)
pub const TABLE_GAP_HEIGHT_LIMIT: u16 = 7;
pub const TIME_LABEL_HEIGHT_LIMIT: u16 = 7;

// Side borders
pub const SIDE_BORDERS: tui::widgets::Borders = tui::widgets::Borders::from_bits_truncate(20);
pub static DEFAULT_TEXT_STYLE: Lazy<tui::style::Style> =
    Lazy::new(|| tui::style::Style::default().fg(tui::style::Color::Gray));
pub static DEFAULT_HEADER_STYLE: Lazy<tui::style::Style> =
    Lazy::new(|| tui::style::Style::default().fg(tui::style::Color::LightBlue));

// Colour profiles
pub static DEFAULT_LIGHT_MODE_COLOUR_PALETTE: Lazy<ConfigColours> = Lazy::new(|| ConfigColours {
    text_color: Some("black".into()),
    border_color: Some("black".into()),
    table_header_color: Some("black".into()),
    widget_title_color: Some("black".into()),
    selected_text_color: Some("white".into()),
    graph_color: Some("black".into()),
    disabled_text_color: Some("gray".into()),
    ram_color: Some("blue".into()),
    swap_color: Some("red".into()),
    arc_color: Some("LightBlue".into()),
    gpu_core_colors: Some(vec![
        "LightGreen".into(),
        "LightCyan".into(),
        "LightRed".into(),
        "Cyan".into(),
        "Green".into(),
        "Blue".into(),
        "Red".into(),
    ]),
    rx_color: Some("blue".into()),
    tx_color: Some("red".into()),
    rx_total_color: Some("LightBlue".into()),
    tx_total_color: Some("LightRed".into()),
    cpu_core_colors: Some(vec![
        "LightMagenta".into(),
        "LightBlue".into(),
        "LightRed".into(),
        "Cyan".into(),
        "Green".into(),
        "Blue".into(),
        "Red".into(),
    ]),
    ..ConfigColours::default()
});

pub static GRUVBOX_COLOUR_PALETTE: Lazy<ConfigColours> = Lazy::new(|| ConfigColours {
    table_header_color: Some("#83a598".into()),
    all_cpu_color: Some("#8ec07c".into()),
    avg_cpu_color: Some("#fb4934".into()),
    cpu_core_colors: Some(vec![
        "#cc241d".into(),
        "#98971a".into(),
        "#d79921".into(),
        "#458588".into(),
        "#b16286".into(),
        "#689d6a".into(),
        "#fe8019".into(),
        "#b8bb26".into(),
        "#fabd2f".into(),
        "#83a598".into(),
        "#d3869b".into(),
        "#d65d0e".into(),
        "#9d0006".into(),
        "#79740e".into(),
        "#b57614".into(),
        "#076678".into(),
        "#8f3f71".into(),
        "#427b58".into(),
        "#d65d03".into(),
        "#af3a03".into(),
    ]),
    ram_color: Some("#8ec07c".into()),
    swap_color: Some("#fabd2f".into()),
    arc_color: Some("#689d6a".into()),
    gpu_core_colors: Some(vec![
        "#d79921".into(),
        "#458588".into(),
        "#b16286".into(),
        "#fe8019".into(),
        "#b8bb26".into(),
        "#cc241d".into(),
        "#98971a".into(),
    ]),
    rx_color: Some("#8ec07c".into()),
    tx_color: Some("#fabd2f".into()),
    rx_total_color: Some("#689d6a".into()),
    tx_total_color: Some("#d79921".into()),
    border_color: Some("#ebdbb2".into()),
    highlighted_border_color: Some("#fe8019".into()),
    disabled_text_color: Some("#665c54".into()),
    text_color: Some("#ebdbb2".into()),
    selected_text_color: Some("#1d2021".into()),
    selected_bg_color: Some("#ebdbb2".into()),
    widget_title_color: Some("#ebdbb2".into()),
    graph_color: Some("#ebdbb2".into()),
    high_battery_color: Some("#98971a".into()),
    medium_battery_color: Some("#fabd2f".into()),
    low_battery_color: Some("#fb4934".into()),
});

pub static GRUVBOX_LIGHT_COLOUR_PALETTE: Lazy<ConfigColours> = Lazy::new(|| ConfigColours {
    table_header_color: Some("#076678".into()),
    all_cpu_color: Some("#8ec07c".into()),
    avg_cpu_color: Some("#fb4934".into()),
    cpu_core_colors: Some(vec![
        "#cc241d".into(),
        "#98971a".into(),
        "#d79921".into(),
        "#458588".into(),
        "#b16286".into(),
        "#689d6a".into(),
        "#fe8019".into(),
        "#b8bb26".into(),
        "#fabd2f".into(),
        "#83a598".into(),
        "#d3869b".into(),
        "#d65d0e".into(),
        "#9d0006".into(),
        "#79740e".into(),
        "#b57614".into(),
        "#076678".into(),
        "#8f3f71".into(),
        "#427b58".into(),
        "#d65d03".into(),
        "#af3a03".into(),
    ]),
    ram_color: Some("#427b58".into()),
    swap_color: Some("#cc241d".into()),
    arc_color: Some("#689d6a".into()),
    gpu_core_colors: Some(vec![
        "#9d0006".into(),
        "#98971a".into(),
        "#d79921".into(),
        "#458588".into(),
        "#b16286".into(),
        "#fe8019".into(),
        "#b8bb26".into(),
    ]),
    rx_color: Some("#427b58".into()),
    tx_color: Some("#cc241d".into()),
    rx_total_color: Some("#689d6a".into()),
    tx_total_color: Some("#9d0006".into()),
    border_color: Some("#3c3836".into()),
    highlighted_border_color: Some("#af3a03".into()),
    disabled_text_color: Some("#d5c4a1".into()),
    text_color: Some("#3c3836".into()),
    selected_text_color: Some("#ebdbb2".into()),
    selected_bg_color: Some("#3c3836".into()),
    widget_title_color: Some("#3c3836".into()),
    graph_color: Some("#3c3836".into()),
    high_battery_color: Some("#98971a".into()),
    medium_battery_color: Some("#d79921".into()),
    low_battery_color: Some("#cc241d".into()),
});

pub static NORD_COLOUR_PALETTE: Lazy<ConfigColours> = Lazy::new(|| ConfigColours {
    table_header_color: Some("#81a1c1".into()),
    all_cpu_color: Some("#88c0d0".into()),
    avg_cpu_color: Some("#8fbcbb".into()),
    cpu_core_colors: Some(vec![
        "#5e81ac".into(),
        "#81a1c1".into(),
        "#d8dee9".into(),
        "#b48ead".into(),
        "#a3be8c".into(),
        "#ebcb8b".into(),
        "#d08770".into(),
        "#bf616a".into(),
    ]),
    ram_color: Some("#88c0d0".into()),
    swap_color: Some("#d08770".into()),
    arc_color: Some("#5e81ac".into()),
    gpu_core_colors: Some(vec![
        "#8fbcbb".into(),
        "#81a1c1".into(),
        "#d8dee9".into(),
        "#b48ead".into(),
        "#a3be8c".into(),
        "#ebcb8b".into(),
        "#bf616a".into(),
    ]),
    rx_color: Some("#88c0d0".into()),
    tx_color: Some("#d08770".into()),
    rx_total_color: Some("#5e81ac".into()),
    tx_total_color: Some("#8fbcbb".into()),
    border_color: Some("#88c0d0".into()),
    highlighted_border_color: Some("#5e81ac".into()),
    disabled_text_color: Some("#4c566a".into()),
    text_color: Some("#e5e9f0".into()),
    selected_text_color: Some("#2e3440".into()),
    selected_bg_color: Some("#88c0d0".into()),
    widget_title_color: Some("#e5e9f0".into()),
    graph_color: Some("#e5e9f0".into()),
    high_battery_color: Some("#a3be8c".into()),
    medium_battery_color: Some("#ebcb8b".into()),
    low_battery_color: Some("#bf616a".into()),
});

pub static NORD_LIGHT_COLOUR_PALETTE: Lazy<ConfigColours> = Lazy::new(|| ConfigColours {
    table_header_color: Some("#5e81ac".into()),
    all_cpu_color: Some("#81a1c1".into()),
    avg_cpu_color: Some("#8fbcbb".into()),
    cpu_core_colors: Some(vec![
        "#5e81ac".into(),
        "#88c0d0".into(),
        "#4c566a".into(),
        "#b48ead".into(),
        "#a3be8c".into(),
        "#ebcb8b".into(),
        "#d08770".into(),
        "#bf616a".into(),
    ]),
    ram_color: Some("#81a1c1".into()),
    swap_color: Some("#d08770".into()),
    arc_color: Some("#5e81ac".into()),
    gpu_core_colors: Some(vec![
        "#8fbcbb".into(),
        "#88c0d0".into(),
        "#4c566a".into(),
        "#b48ead".into(),
        "#a3be8c".into(),
        "#ebcb8b".into(),
        "#bf616a".into(),
    ]),
    rx_color: Some("#81a1c1".into()),
    tx_color: Some("#d08770".into()),
    rx_total_color: Some("#5e81ac".into()),
    tx_total_color: Some("#8fbcbb".into()),
    border_color: Some("#2e3440".into()),
    highlighted_border_color: Some("#5e81ac".into()),
    disabled_text_color: Some("#d8dee9".into()),
    text_color: Some("#2e3440".into()),
    selected_text_color: Some("#f5f5f5".into()),
    selected_bg_color: Some("#5e81ac".into()),
    widget_title_color: Some("#2e3440".into()),
    graph_color: Some("#2e3440".into()),
    high_battery_color: Some("#a3be8c".into()),
    medium_battery_color: Some("#ebcb8b".into()),
    low_battery_color: Some("#bf616a".into()),
});

// Help text
pub const HELP_CONTENTS_TEXT: [&str; 10] = [
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
pub const GENERAL_HELP_TEXT: [&str; 32] = [
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

pub const CPU_HELP_TEXT: [&str; 2] = [
    "2 - CPU widget",
    "Mouse scroll     Scrolling over an CPU core/average shows only that entry on the chart",
];

pub const PROCESS_HELP_TEXT: [&str; 15] = [
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
];

pub const SEARCH_HELP_TEXT: [&str; 48] = [
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
    "read, r/s        ex: read >= 1 b",
    "write, w/s       ex: write <= 1 tb",
    "tread, t.read    ex: tread = 1",
    "twrite, t.write  ex: twrite = 1",
    "user            ex: user = root",
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
    "5 - Sort widget",
    "Down, 'j'        Scroll down in list",
    "Up, 'k'          Scroll up in list",
    "Mouse scroll     Scroll through sort widget",
    "Esc              Close the sort widget",
    "Enter            Sort by current selected column",
];

pub const TEMP_HELP_WIDGET: [&str; 3] = [
    "6 - Temperature widget",
    "'s'              Sort by sensor name, press again to reverse",
    "'t'              Sort by temperature, press again to reverse",
];

pub const DISK_HELP_WIDGET: [&str; 9] = [
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

pub const BATTERY_HELP_TEXT: [&str; 3] = [
    "8 - Battery widget",
    "Left             Go to previous battery",
    "Right            Go to next battery",
];

pub const BASIC_MEM_HELP_TEXT: [&str; 2] = [
    "9 - Basic memory widget",
    "%                Toggle between values and percentages for memory usage",
];

pub const HELP_TEXT: [&[&str]; HELP_CONTENTS_TEXT.len()] = [
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

// TODO: Eventually deprecate this.
pub const CONFIG_TEXT: &str = r##"# This is a default config file for bottom.  All of the settings are commented
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
# Hides the time scale.
#hide_time = false
# Override layout default widget
#default_widget_type = "proc"
#default_widget_count = 1
# Expand selected widget upon starting the app
#expanded_on_startup = true
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
# Built-in themes.  Valid values are "default", "default-light", "gruvbox", "gruvbox-light", "nord", "nord-light"
#color = "default"
# Show memory values in the processes widget as values by default
#mem_as_value = false
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
# Shows GPU(s) memory
#enable_gpu_memory = false
# How much data is stored at once in terms of time.
#retention = "10m"

# These are all the components that support custom theming.  Note that colour support
# will depend on terminal support.

#[colors] # Uncomment if you want to use custom colors
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
# Represents the colour ARC will use in the memory legend and graph.
#arc_color="LightCyan"
# Represents the colour the GPU will use in the memory legend and graph.
#gpu_core_colors=["LightGreen", "LightBlue", "LightRed", "Cyan", "Green", "Blue", "Red"]
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
#high_battery_color="green"
#medium_battery_color="yellow"
#low_battery_color="red"

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


# Filters - you can hide specific temperature sensors, network interfaces, and disks using filters.  This is admittedly
# a bit hard to use as of now, and there is a planned in-app interface for managing this in the future:
#[disk_filter]
#is_list_ignored = true
#list = ["/dev/sda\\d+", "/dev/nvme0n1p2"]
#regex = true
#case_sensitive = false
#whole_word = false

#[mount_filter]
#is_list_ignored = true
#list = ["/mnt/.*", "/boot"]
#regex = true
#case_sensitive = false
#whole_word = false

#[temp_filter]
#is_list_ignored = true
#list = ["cpu", "wifi"]
#regex = false
#case_sensitive = false
#whole_word = false

#[net_filter]
#is_list_ignored = true
#list = ["virbr0.*"]
#regex = true
#case_sensitive = false
#whole_word = false
"##;

pub const CONFIG_TOP_HEAD: &str = r##"# This is bottom's config file.
# Values in this config file will change when changed in the interface.
# You can also manually change these values.
# Be aware that contents of this file will be overwritten if something is
# changed in the application; you can disable writing via the
# --no_write flag or no_write config option.

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn help_menu_matches_entry_len() {
        assert_eq!(
            HELP_CONTENTS_TEXT.len(),
            HELP_TEXT.len(),
            "the two should be equal, or this test should be updated"
        )
    }
}
