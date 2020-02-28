pub const STALE_MAX_MILLISECONDS : u128 = 60 * 1000; // How long to store data.
pub const TIME_STARTS_FROM : u64 = 60 * 1000;
pub const TICK_RATE_IN_MILLISECONDS : u64 = 200; // How fast the screen refreshes
pub const DEFAULT_REFRESH_RATE_IN_MILLISECONDS : u128 = 1000;
pub const MAX_KEY_TIMEOUT_IN_MILLISECONDS : u128 = 1000;
pub const NUM_COLOURS : i32 = 256;

// Config and flags
pub const DEFAULT_UNIX_CONFIG_FILE_PATH : &str = ".config/bottom/bottom.toml";
pub const DEFAULT_WINDOWS_CONFIG_FILE_PATH : &str = "bottom/bottom.toml";

// Help text
pub const GENERAL_HELP_TEXT : [&str; 15] = [
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

pub const PROCESS_HELP_TEXT : [&str; 8] = [
	"Process Keybindings\n\n",
	"dd             Kill the highlighted process\n",
	"c              Sort by CPU usage\n",
	"m              Sort by memory usage\n",
	"p              Sort by PID\n",
	"n              Sort by process name\n",
	"Tab            Group together processes with the same name\n",
	"Ctrl-f, /      Open up the search widget\n",
];

pub const SEARCH_HELP_TEXT : [&str; 13] = [
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
