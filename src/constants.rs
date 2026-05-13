//! A bunch of constants used throughout the application.
//!
//! FIXME: Move these to where it makes more sense.

// Default widget ID
pub const DEFAULT_WIDGET_ID: u64 = 56709;

// Limits for when we should stop showing table gaps/labels (anything less means
// not shown)
pub const TABLE_GAP_HEIGHT_LIMIT: u16 = 7;

// Help text
const HELP_CONTENTS_TEXT: [&str; 12] = [
    "Scroll to browse or press the number key to go to the corresponding help menu section:",
    "1 - General",
    "2 - CPU widget",
    "3 - Process widget",
    "4 - Process search widget",
    "5 - Process sort widget",
    "6 - Temperature widget",
    "7 - Disk widget",
    "8 - Battery widget",
    "9 - Basic memory widget",
    "",
    "Press 'Ctrl-f' or '/' to search for a keyword in the help text.",
];

// TODO [Help]: Move to using tables for easier formatting?
pub(crate) const GENERAL_HELP_TEXT: [&str; 24] = [
    "1 - General",
    "q, Ctrl-c               Quit",
    "Esc                     Close dialog windows, search, widgets, or exit expanded mode",
    "Ctrl-r                  Reset display and any collected data",
    "f                       Freeze/unfreeze updating with new data",
    "Shift/Ctrl-Left, H, A   Move widget selection left",
    "Shift/Ctrl-Right, L, D  Move widget selection right",
    "Shift/Ctrl-Up, K, W     Move widget selection up",
    "Shift/Ctrl-Down, J, S   Move widget selection down",
    "Left, h                 Move left within widget",
    "Down, j                 Move down within widget",
    "Up, k                   Move up within widget",
    "Right, l                Move right within widget",
    "?                       Open help menu",
    "gg                      Jump to the first entry",
    "G                       Jump to the last entry",
    "e                       Toggle expanding the currently selected widget",
    "+                       Zoom in on chart (decrease time range)",
    "-                       Zoom out on chart (increase time range)",
    "=                       Reset zoom",
    "PgUp, PgDown            Scroll up/down a table by a page",
    "Ctrl-u, Ctrl-d          Scroll up/down a table by half a page",
    "Mouse scroll            Scroll through the tables or zoom in/out of charts by scrolling up/down",
    "Mouse click             Selects the clicked widget, table entry, dialog option, or tab",
];

const CPU_HELP_TEXT: [&str; 2] = [
    "2 - CPU widget",
    "Mouse scroll            Scrolling over a CPU core/average shows only that entry on the chart",
];

const PROCESS_HELP_TEXT: [&str; 20] = [
    "3 - Process widget",
    "dd, F9, Delete          Kill the selected process",
    "c                       Sort by CPU usage, press again to reverse",
    "m                       Sort by memory usage, press again to reverse",
    "p                       Sort by PID name, press again to reverse",
    "n                       Sort by process name, press again to reverse",
    "Tab                     Group/un-group processes with the same name",
    "Ctrl-f, /               Open process search widget",
    "P                       Toggle between showing the full command or just the process name",
    "s, F6                   Open process sort widget",
    "I                       Invert current sort",
    "%                       Toggle between values and percentages for memory usage",
    "t, F5                   Toggle tree mode",
    "Right                   Collapse a branch while in tree mode",
    "Left                    Expand a branch while in tree mode",
    "+, -, click, Space      Toggle whether a branch is expanded or collapsed in tree mode",
    "click on header         Sorts the entries by that column, click again to invert the sort",
    "C                       Sort by GPU usage, press again to reverse",
    "M                       Sort by GPU memory usage, press again to reverse",
    "z                       Toggle the display of kernel threads",
];

const SEARCH_HELP_TEXT: [&str; 53] = [
    "4 - Process search widget",
    "Esc                     Close the search widget (retains the filter)",
    "Ctrl-a                  Skip to the start of the search query",
    "Ctrl-e                  Skip to the end of the search query",
    "Ctrl-u                  Clear the current search query",
    "Ctrl-w                  Delete a word behind the cursor",
    "Ctrl-h                  Delete the character behind the cursor",
    "Backspace               Delete the character behind the cursor",
    "Delete                  Delete the character at the cursor",
    "Alt-c, F1               Toggle matching case",
    "Alt-w, F2               Toggle matching the entire word",
    "Alt-r, F3               Toggle using regex",
    "Left, Alt-h             Move cursor left",
    "Right, Alt-l            Move cursor right",
    "",
    "Supported search types:",
    "<by name/cmd>           ex: btm",
    "pid                     ex: pid 825",
    "cpu, cpu%               ex: cpu > 4.2",
    "mem, mem%               ex: mem < 4.2",
    "memb                    ex: memb < 100 kb",
    "read, r/s, rps          ex: read >= 1 b",
    "write, w/s, wps         ex: write <= 1 tb",
    "tread, t.read           ex: tread = 1",
    "twrite, t.write         ex: twrite = 1",
    "user                    ex: user = root",
    "state                   ex: state = running",
    "gpu%                    ex: gpu% < 4.2",
    "gmem                    ex: gmem < 100 kb",
    "gmem%                   ex: gmem% < 4.2",
    "",
    "Comparison operators:",
    "=                       ex: cpu = 1",
    "!=                      ex: cpu != 1",
    ">                       ex: cpu > 1",
    "<                       ex: cpu < 1",
    ">=                      ex: cpu >= 1",
    "<=                      ex: cpu <= 1",
    "",
    "Logical operators:",
    "and, &&, <Space>        ex: btm and cpu > 1 and mem > 1",
    "or, ||                  ex: btm or firefox",
    "!                       ex: !firefox, !(cpu > 5 or btm)",
    "",
    "Supported units:",
    "B                       ex: read > 1 b",
    "KB                      ex: read > 1 kb",
    "MB                      ex: read > 1 mb",
    "TB                      ex: read > 1 tb",
    "KiB                     ex: read > 1 kib",
    "MiB                     ex: read > 1 mib",
    "GiB                     ex: read > 1 gib",
    "TiB                     ex: read > 1 tib",
];

const SORT_HELP_TEXT: [&str; 6] = [
    "5 - Sort widget",
    "Down, 'j'               Scroll down in list",
    "Up, 'k'                 Scroll up in list",
    "Mouse scroll            Scroll through sort widget",
    "Esc                     Close the sort widget",
    "Enter                   Sort by current selected column",
];

const TEMP_HELP_WIDGET: [&str; 3] = [
    "6 - Temperature widget",
    "'s'                     Sort by sensor name, press again to reverse",
    "'t'                     Sort by temperature, press again to reverse",
];

const DISK_HELP_WIDGET: [&str; 9] = [
    "7 - Disk widget",
    "'d'                     Sort by disk name, press again to reverse",
    "'m'                     Sort by disk mount, press again to reverse",
    "'u'                     Sort by disk usage, press again to reverse",
    "'n'                     Sort by disk free space, press again to reverse",
    "'t'                     Sort by total disk space, press again to reverse",
    "'p'                     Sort by disk usage percentage, press again to reverse",
    "'r'                     Sort by disk read activity, press again to reverse",
    "'w'                     Sort by disk write activity, press again to reverse",
];

const BATTERY_HELP_TEXT: [&str; 3] = [
    "8 - Battery widget",
    "Left                    Go to previous battery",
    "Right                   Go to next battery",
];

const BASIC_MEM_HELP_TEXT: [&str; 2] = [
    "9 - Basic memory widget",
    "%                       Toggle between values and percentages for memory usage",
];

/// The number of help sections.
const HELP_SECTIONS: usize = 10;

// TODO: Add temp graph help section.
pub(crate) const HELP_TEXT: [&[&str]; HELP_SECTIONS] = [
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

# Whether to use a dedicated row for the average cpu entry
#average_cpu_row = false

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

# Whether to group processes with the same name together by default. Doesn't do anything
# if tree is set to true or --tree is set.
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

# Controls the gap between table headers and data rows.
# Options: "none", "space" (default), "line"
#table_gap = "space"

# Show the battery widgets
#battery = false

# Disable mouse clicks
#disable_click = false

# Disable keyboard shortcuts
#disable_keys = false

# Show memory values in the processes widget as values by default
#process_memory_as_value = false

# Show tree mode by default in the processes widget.
#tree = false

# Shows an indicator in table widgets tracking where in the list you are.
#show_table_scroll_position = false

# Show a scroll bar on the right edge of table widgets.
#show_table_scroll_bar = false

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

# Prevents performing any actions that affect the system (e.g. stopping processes).
#read_only = false

# Hide kernel threads from being shown.
#hide_k_threads = false

# Hide GPU(s) information
#disable_gpu = false

# Shows cache and buffer memory
#enable_cache_memory = false

# Subtract freeable ARC from memory usage
#free_arc = false

# How much data is stored at once in terms of time.
#retention = "10m"

# Where to place the legend for the memory widget. One of "none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right".
#memory_legend = "top-right"

# Where to place the legend for the network widget. One of "none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right".
#network_legend = "top-right"


# Processes widget configuration
#[processes]
# The columns shown by the process widget. The following columns are supported (the GPU columns are only available if the GPU feature is enabled when built):
# PID, Name, CPU%, Mem%, R/s, W/s, T.Read, T.Write, User, State, Time, GMem%, GPU%, Nice, Priority
#columns = ["PID", "Name", "CPU%", "Mem%", "Virt", "R/s", "W/s", "T.Read", "T.Write", "User", "State", "GMem%", "GPU%", "Priority"]

# The default sort column when bottom starts. Accepts any of the column names above.
# If unset, defaults to CPU%.
#default_sort = "CPU%"

# Gather process child thread information
#get_threads = false

# Hide kernel threads from being shown. Linux only.
#hide_k_threads = false

# Collapse the process tree by default when tree mode is set.
#tree_collapse = false

# Shows the full command name instead of the process name by default.
#process_command = false

# Disable the advanced kill dialog and just show the basic one with no options. Only available on Linux, macOS, and FreeBSD.
#disable_advanced_kill = false

# Defaults to showing process memory usage by value.
#default_memory_value = false

# Groups processes with the same name by default. No effect if tree is set.
#default_grouped = false

# Enables regex by default while searching.
#regex = false

# Enables case sensitivity by default when searching.
#case_sensitive = false

# Enables whole-word matching by default while searching.
#whole_word = false

# Makes the process widget use tree mode by default.
#default_tree = false

# Calculates process CPU usage as a percentage of current usage rather than total usage.
#current_usage = false

# Show process CPU% usage without averaging over the number of CPU cores.
#unnormalized_cpu = false


# CPU widget configuration
#[cpu]
# One of "all" (default), "average"/"avg"
#default = "average"

# Whether to show a decimal place for CPU usage values.
#show_decimal = false


# Disk widget configuration
#[disk]

# The columns shown by the process widget. The following columns are supported:
# Disk, Mount, Used, Free, Total, Used%, Free%, R/s, W/s
#columns = ["Disk", "Mount", "Used", "Free", "Total", "Used%", "R/s", "W/s"]

# The default sort type. Can be one of the following:
# Disk, Mount, Used, Free, Total, Used%, Free%, R/s, W/s
#
# Defaults to "Disk".
#default_sort = "Disk"

# By default, there are no disk name filters enabled. These can be turned on to filter out specific data entries if you
# don't want to see them. An example use case is provided below.
#[disk.name_filter]
# Whether to ignore any matches. Defaults to true.
#is_list_ignored = true

# A list of filters to try and match.
#list = ["/dev/sda\\d+", "/dev/nvme0n1p2"]

# Whether to use regex. Defaults to false.
#regex = true

# Whether to be case-sensitive. Defaults to false.
#case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
#whole_word = false

# By default, there are no mount name filters enabled. An example use case is provided below.
#[disk.mount_filter]
# Whether to ignore any matches. Defaults to true.
#is_list_ignored = true

# A list of filters to try and match.
#list = ["/mnt/.*", "/boot"]

# Whether to use regex. Defaults to false.
#regex = true

# Whether to be case-sensitive. Defaults to false.
#case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
#whole_word = false


# Temperature widget configuration
#[temperature]

# The default sort type. Can be one of the following:
# Temp, Temperature, Sensor
#
# Defaults to "Sensor".
#default_sort = "Sensor"

# By default, there are no temperature sensor filters enabled. An example use case is provided below.
#[temperature.sensor_filter]
# Whether to ignore any matches. Defaults to true.
#is_list_ignored = true

# A list of filters to try and match.
#list = ["cpu", "wifi"]

# Whether to use regex. Defaults to false.
#regex = false

# Whether to be case-sensitive. Defaults to false.
#case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
#whole_word = false


# Temperature graph widget configuration
#[temperature_graph]

# Where to place the legend for the temperature graph widget. One of "none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right".
#legend_position = "top-right"

# An upper temperature value for the graph; entries higher than this will be hidden. If not set,
# there is no limit. Is in the unit of `temperature_type`.
#max_temp = 100.0

# By default, there are no temperature sensor filters enabled. An example use case is provided below.
#[temperature_graph.sensor_filter]
# Whether to ignore any matches. Defaults to true.
#is_list_ignored = true

# A list of filters to try and match.
#list = ["cpu", "wifi"]

# Whether to use regex. Defaults to false.
#regex = false

# Whether to be case-sensitive. Defaults to false.
#case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
#whole_word = false


# Network widget configuration
#[network_graph]
# By default, there are no network interface filters enabled. An example use case is provided below.
#[network_graph.interface_filter]
# Whether to ignore any matches. Defaults to true.
#is_list_ignored = true

# A list of filters to try and match.
#list = ["virbr0.*"]

# Whether to use regex. Defaults to false.
#regex = true

# Whether to be case-sensitive. Defaults to false.
#case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
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

# Styling options. You can control things like colour choices, text styling (bold, italic, etc), and more.
# Note that any setting that uses the word "colour" can be substituted for "color" and it will still work fine.

#[styles.cpu]
#all_entry_colour = "green"
#avg_entry_colour = "red"
#cpu_core_colours = ["light magenta", "light yellow", "light cyan", "light green", "light blue", "cyan", "green", "blue"]

#[styles.temp_graph]
#temp_graph_colour_styles = ["light magenta", "light yellow", "light cyan", "light green", "light blue", "cyan", "green", "blue"]

#[styles.memory]
#ram_colour = "light magenta"
#cache_colour = "light red"
#swap_colour = "light yellow"
#arc_colour = "light cyan"
#gpu_colours = ["light blue", "light red", "cyan", "green", "blue", "red"]

#[styles.network]
#rx_colour = "light magenta"
#tx_colour = "light yellow"
#rx_total_colour = "light cyan"
#tx_total_colour = "light green"

#[styles.battery]
#high_battery_colour = "green"
#medium_battery_colour = "yellow"
#low_battery_colour = "red"

#[styles.tables]
#headers = {colour = "light blue", bold = true}

#[styles.graphs]
#graph_colour = "gray"
#legend_text = {colour = "gray"}

#[styles.widgets]
#border_colour = "gray"
#selected_border_colour = "light blue"
#widget_title = {colour = "gray"}
#text = {colour = "gray"}
#selected_text = {colour = "black", bg_colour = "light blue"}
#disabled_text = {colour = "dark gray"}
# Disabled by default
#bg_colour = "black"
# Only on Linux
#thread_text = {colour = "green"}

# Layout - layouts follow a pattern like this:
# [[row]] represents a row in the application.
# [[row.child]] represents either a widget or a column.
# [[row.child.child]] represents a widget.
#
# All widgets must have the type value set to one of ["cpu", "mem", "proc", "net", "temp", "temp_graph", "disk", "empty"].
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
        // Subtract 2 to account for the extra newline + search instructions at the bottom.
        const HELP_CONTENTS_TEXT_LEN: usize = HELP_CONTENTS_TEXT.len() - 2;

        assert_eq!(
            HELP_CONTENTS_TEXT_LEN,
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

    /// Checks that the default config is valid.
    #[test]
    #[cfg(feature = "default")]
    fn check_default_config() {
        use regex::Regex;

        use crate::options::Config;

        // Trim off the starting comment if it's a "#" directly following an
        // alphabetical character or '['.
        let default_config = Regex::new(r"(?m)^#([a-zA-Z\[])")
            .unwrap()
            .replace_all(CONFIG_TEXT, "$1");

        // Then, trim off anything that has more than 2 spaces + alphabetical character
        // or '[' following a "#".
        let default_config = Regex::new(r"(?m)^#(\s\s+)([a-zA-Z\[])")
            .unwrap()
            .replace_all(&default_config, "$2");

        let _config: Config =
            toml_edit::de::from_str(&default_config).expect("can parse default config");

        // TODO: Check this.
        // assert_eq!(config, Config::default());
    }
}
