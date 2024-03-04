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

// Limits for when we should stop showing table gaps/labels (anything less means not shown)
pub const TABLE_GAP_HEIGHT_LIMIT: u16 = 7;
pub const TIME_LABEL_HEIGHT_LIMIT: u16 = 7;

// Side borders
pub const SIDE_BORDERS: Borders = Borders::LEFT.union(Borders::RIGHT);

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

pub const PROCESS_HELP_TEXT: [&str; 17] = [
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

pub const SEARCH_HELP_TEXT: [&str; 51] = [
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
pub const DEFAULT_LAYOUT: &str = r#"
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

pub const DEFAULT_BATTERY_LAYOUT: &str = r#"
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

    /// This test exists because previously, [`SIDE_BORDERS`] was set incorrectly after I moved from
    /// tui-rs to ratatui.
    #[test]
    fn assert_side_border_bits_match() {
        assert_eq!(
            SIDE_BORDERS,
            Borders::ALL.difference(Borders::TOP.union(Borders::BOTTOM))
        )
    }
}
