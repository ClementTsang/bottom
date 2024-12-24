//! Argument parsing via clap.
//!
//! Note that you probably want to keep this as a single file so the build
//! script doesn't trip all over itself.

// TODO: New sections are misaligned! See if we can get that fixed.

use std::path::PathBuf;

use clap::{builder::PossibleValue, *};
use indoc::indoc;

const TEMPLATE: &str = indoc! {
    "{name} {version}
    {author}

    {about}

    {usage-heading} {usage}

    {all-args}"
};

const USAGE: &str = "btm [OPTIONS]";

const VERSION: &str = match option_env!("NIGHTLY_VERSION") {
    Some(nightly_version) => nightly_version,
    None => crate_version!(),
};

const CHART_WIDGET_POSITIONS: [&str; 9] = [
    "none",
    "top-left",
    "top",
    "top-right",
    "left",
    "right",
    "bottom-left",
    "bottom",
    "bottom-right",
];

/// Represents the arguments that can be passed in to bottom.
#[derive(Parser, Debug)]
#[command(
    name = crate_name!(),
    version = VERSION,
    author = crate_authors!(),
    about = crate_description!(),
    disable_help_flag = true,
    disable_version_flag = true,
    color = ColorChoice::Auto,
    help_template = TEMPLATE,
    override_usage = USAGE,
)]
pub struct BottomArgs {
    #[command(flatten)]
    pub general: GeneralArgs,

    #[command(flatten)]
    pub process: ProcessArgs,

    #[command(flatten)]
    pub temperature: TemperatureArgs,

    #[command(flatten)]
    pub cpu: CpuArgs,

    #[command(flatten)]
    pub memory: MemoryArgs,

    #[command(flatten)]
    pub network: NetworkArgs,

    #[cfg(feature = "battery")]
    #[command(flatten)]
    pub battery: BatteryArgs,

    #[cfg(feature = "gpu")]
    #[command(flatten)]
    pub gpu: GpuArgs,

    #[command(flatten)]
    pub style: StyleArgs,

    #[command(flatten)]
    pub other: OtherArgs,
}

/// General arguments/config options.
#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "General Options", rename_all = "snake_case")]
pub struct GeneralArgs {
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Temporarily shows the time scale in graphs.",
        long_help = "Automatically hides the time scale in graphs after being shown for a brief moment when zoomed \
                in/out. If time is disabled using --hide_time then this will have no effect."
    )]
    pub autohide_time: bool,

    #[arg(
        short = 'b',
        long,
        action = ArgAction::SetTrue,
        help = "Hides graphs and uses a more basic look.",
        long_help = "Hides graphs and uses a more basic look, largely inspired by htop's design."
    )]
    pub basic: bool,

    #[arg(
        short = 'C',
        long,
        value_name = "PATH",
        value_hint = ValueHint::AnyPath,
        help = "Sets the location of the config file.",
        long_help = "Sets the location of the config file. Expects a config file in the TOML format. \
                    If it doesn't exist, a default config file is created at the path. If no path is provided, \
                    the default config location will be used."
    )]
    pub config_location: Option<PathBuf>,

    #[arg(
        short = 't',
        long,
        value_name = "TIME",
        help = "Default time value for graphs.",
        long_help = "Default time value for graphs. Either a number in milliseconds or a 'human duration' \
                    (e.g. 60s, 10m). Defaults to 60s, must be at least 30s."
    )]
    pub default_time_value: Option<String>,

    // TODO: Charts are broken in the manpage
    #[arg(
        long,
        requires_all = ["default_widget_type"],
        value_name = "N",
        help = "Sets the N'th selected widget type as the default.",
        long_help = indoc! {
            "Sets the N'th selected widget type to use as the default widget. Requires 'default_widget_type' to also be \
            set, and defaults to 1.

            This reads from left to right, top to bottom. For example, suppose we have a layout that looks like:
            +-------------------+-----------------------+
            |      CPU (1)      |        CPU (2)        |
            +---------+---------+-------------+---------+
            | Process | CPU (3) | Temperature | CPU (4) |
            +---------+---------+-------------+---------+

            And we set our default widget type to 'CPU'. If we set '--default_widget_count 1', then it would use the \
            CPU (1) as the default widget. If we set '--default_widget_count 3', it would use CPU (3) as the default \
            instead."
        }
    )]
    pub default_widget_count: Option<u64>,

    #[arg(
        long,
        value_name = "WIDGET",
        help = "Sets the default widget type. Use --help for more info.",
        long_help = indoc!{
            "Sets which widget type to use as the default widget. For the default \
            layout, this defaults to the 'process' widget. For a custom layout, it defaults \
            to the first widget it sees.

            For example, suppose we have a layout that looks like:
            +-------------------+-----------------------+
            |      CPU (1)      |        CPU (2)        |
            +---------+---------+-------------+---------+
            | Process | CPU (3) | Temperature | CPU (4) |
            +---------+---------+-------------+---------+

            Then, setting '--default_widget_type temperature' will make the temperature widget selected by default."
        },
        value_parser = [
            "cpu",
            "mem",
            "net",
            "network",
            "proc",
            "process",
            "processes",
            "temp",
            "temperature",
            "disk",
            #[cfg(feature = "battery")]
            "batt",
            #[cfg(feature = "battery")]
            "battery",
        ],
    )]
    pub default_widget_type: Option<String>,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Disables mouse clicks.",
        long_help = "Disables mouse clicks from interacting with bottom."
    )]
    pub disable_click: bool,

    // TODO: Change this to accept a string with the type of marker.
    #[arg(
        short = 'm',
        long,
        action = ArgAction::SetTrue,
        help = "Uses a dot marker for graphs.",
        long_help = "Uses a dot marker for graphs as opposed to the default braille marker."
    )]
    pub dot_marker: bool,

    #[arg(
        short = 'e',
        long,
        action = ArgAction::SetTrue,
        help = "Expand the default widget upon starting the app.",
        long_help = "Expand the default widget upon starting the app. This flag has no effect in basic mode (--basic)."
    )]
    pub expanded: bool,

    #[arg(long, action = ArgAction::SetTrue, help = "Hides spacing between table headers and entries.")]
    pub hide_table_gap: bool,

    #[arg(long, action = ArgAction::SetTrue, help = "Hides the time scale from being shown.")]
    pub hide_time: bool,

    #[arg(
        short = 'r',
        long,
        value_name = "TIME",
        help = "Sets how often data is refreshed.",
        long_help = "Sets how often data is refreshed. Either a number in milliseconds or a 'human duration' \
                    (e.g. 1s, 1m). Defaults to 1s, must be at least 250ms. Smaller values may result in \
                    higher system resource usage."
    )]
    pub rate: Option<String>,

    #[arg(
        long,
        value_name = "TIME",
        help = "How far back data will be stored up to.",
        long_help = "How far back data will be stored up to. Either a number in milliseconds or a 'human duration' \
                    (e.g. 10m, 1h). Defaults to 10 minutes, and must be at least  1 minute. Larger values \
                    may result in higher memory usage."
    )]
    pub retention: Option<String>,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Shows the list scroll position tracker in the widget title for table widgets."
    )]
    pub show_table_scroll_position: bool,

    #[arg(
        short = 'd',
        long,
        value_name = "TIME",
        help = "The amount of time changed upon zooming.",
        long_help = "The amount of time changed when zooming in/out. Takes a number in \
                    milliseconds or a human duration (e.g. 30s). The minimum is 1s, and \
                    defaults to 15s."
    )]
    pub time_delta: Option<String>,
}

/// Process arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Process Options", rename_all = "snake_case")]
pub struct ProcessArgs {
    #[arg(
        short = 'S',
        long,
        action = ArgAction::SetTrue,
        help = "Enables case sensitivity by default.",
        long_help = "Enables case sensitivity by default when searching for a process."
    )]
    pub case_sensitive: bool,

    // TODO: Rename this.
    #[arg(
        short = 'u',
        long,
        action = ArgAction::SetTrue,
        help = "Calculates process CPU usage as a percentage of current usage rather than total usage."
    )]
    pub current_usage: bool,

    // TODO: Disable this on Windows?
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Hides additional stopping options Unix-like systems.",
        long_help = "Hides additional stopping options Unix-like systems. Signal 15 (TERM) will be sent when \
                    stopping a process."
    )]
    pub disable_advanced_kill: bool,

    #[arg(
        short = 'g',
        long,
        action = ArgAction::SetTrue,
        help = "Groups processes with the same name by default."
    )]
    pub group_processes: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Defaults to showing process memory usage by value.",
        long_help = "Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage."
    )]
    pub process_memory_as_value: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Shows the full command name instead of the process name by default."
    )]
    pub process_command: bool,

    #[arg(short = 'R', long, action = ArgAction::SetTrue, help = "Enables regex by default while searching.")]
    pub regex: bool,

    #[arg(
        short = 'T',
        long,
        action = ArgAction::SetTrue,
        help = "Makes the process widget use tree mode by default."
    )]
    pub tree: bool,

    #[arg(
        short = 'n',
        long,
        action = ArgAction::SetTrue,
        help = "Show process CPU% usage without averaging over the number of CPU cores."
    )]
    pub unnormalized_cpu: bool,

    #[arg(
        short = 'W',
        long,
        action = ArgAction::SetTrue,
        help = "Enables whole-word matching by default while searching."
    )]
    pub whole_word: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Collapse process tree by default."
    )]
    pub tree_collapse: bool,
}

/// Temperature arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Temperature Options", rename_all = "snake_case")]
#[group(id = "temperature_unit", multiple = false)]
pub struct TemperatureArgs {
    #[arg(
        short = 'c',
        long,
        action = ArgAction::SetTrue,
        group = "temperature_unit",
        help = "Use Celsius as the temperature unit. Default.",
        long_help = "Use Celsius as the temperature unit. This is the default option."
    )]
    pub celsius: bool,

    #[arg(
        short = 'f',
        long,
        action = ArgAction::SetTrue,
        group = "temperature_unit",
        help = "Use Fahrenheit as the temperature unit."
    )]
    pub fahrenheit: bool,

    #[arg(
        short = 'k',
        long,
        action = ArgAction::SetTrue,
        group = "temperature_unit",
        help = "Use Kelvin as the temperature unit."
    )]
    pub kelvin: bool,
}

/// The default selection of the CPU widget. If the given selection is invalid,
/// we will fall back to all.
#[derive(Clone, Copy, Debug, Default)]
pub enum CpuDefault {
    #[default]
    All,
    Average,
}

impl ValueEnum for CpuDefault {
    fn value_variants<'a>() -> &'a [Self] {
        &[CpuDefault::All, CpuDefault::Average]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            CpuDefault::All => Some(PossibleValue::new("all")),
            CpuDefault::Average => Some(PossibleValue::new("avg").alias("average")),
        }
    }
}

/// CPU arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "CPU Options", rename_all = "snake_case")]
pub struct CpuArgs {
    // TODO: Maybe rename this or fix this? Should this apply to all "left legends"?
    #[arg(
        short = 'l',
        long,
        action = ArgAction::SetTrue,
        help = "Puts the CPU chart legend on the left side."
    )]
    pub cpu_left_legend: bool,

    #[arg(
        long,
        help = "Sets which CPU entry type is selected by default.",
        value_name = "ENTRY",
        value_parser = value_parser!(CpuDefault),
    )]
    pub default_cpu_entry: Option<CpuDefault>,

    #[arg(short = 'a', long, action = ArgAction::SetTrue, help = "Hides the average CPU usage entry.")]
    pub hide_avg_cpu: bool,
}

/// Memory argument/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Memory Options", rename_all = "snake_case")]
pub struct MemoryArgs {
    #[arg(
        long,
        value_parser = CHART_WIDGET_POSITIONS,
        value_name = "POSITION",
        ignore_case = true,
        help = "Where to place the legend for the memory chart widget.",
    )]
    pub memory_legend: Option<String>,

    #[cfg(not(target_os = "windows"))]
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Enables collecting and displaying cache and buffer memory."
    )]
    pub enable_cache_memory: bool,
}

/// Network arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Network Options", rename_all = "snake_case")]
pub struct NetworkArgs {
    #[arg(
        long,
        value_parser = CHART_WIDGET_POSITIONS,
        value_name = "POSITION",
        ignore_case = true,
        help = "Where to place the legend for the network chart widget.",
    )]
    pub network_legend: Option<String>,

    // TODO: Rename some of these to remove the network prefix for serde.
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Displays the network widget using bytes.",
        long_help = "Displays the network widget using bytes. Defaults to bits."
    )]
    pub network_use_bytes: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Displays the network widget with binary prefixes.",
        long_help = "Displays the network widget with binary prefixes (e.g. kibibits, mebibits) rather than a decimal \
                    prefixes (e.g. kilobits, megabits). Defaults to decimal prefixes."
    )]
    pub network_use_binary_prefix: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Displays the network widget with a log scale.",
        long_help = "Displays the network widget with a log scale. Defaults to a non-log scale."
    )]
    pub network_use_log: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "(DEPRECATED) Uses a separate network legend.",
        long_help = "(DEPRECATED) Uses separate network widget legend. This display is not tested and may be broken."
    )]
    pub use_old_network_legend: bool,
}

/// Battery arguments/config options.
#[cfg(feature = "battery")]
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Battery Options", rename_all = "snake_case")]
pub struct BatteryArgs {
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Shows the battery widget in non-custom layouts.",
        long_help = "Shows the battery widget in default or basic mode, if there is as battery available. This \
                    has no effect on custom layouts; if the battery widget is desired for a custom layout, explicitly \
                    specify it."
    )]
    pub battery: bool,
}

/// GPU arguments/config options.
#[cfg(feature = "gpu")]
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "GPU Options", rename_all = "snake_case")]
pub struct GpuArgs {
    #[arg(long, action = ArgAction::SetTrue, help = "Disable collecting and displaying NVIDIA and AMD GPU information.")]
    pub disable_gpu: bool,
}

/// Style arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Style Options", rename_all = "snake_case")]
pub struct StyleArgs {
    #[arg(
        long,
        value_name = "SCHEME",
        value_parser = [
            "default",
            "default-light",
            "gruvbox",
            "gruvbox-light",
            "nord",
            "nord-light",
        ],
        hide_possible_values = true,
        help = indoc! {
            "Use a built-in color theme, use '--help' for info on the colors. [possible values: default, default-light, gruvbox, gruvbox-light, nord, nord-light]",
        },
        long_help = indoc! {
            "Use a pre-defined color theme. Currently supported themes are:
            - default
            - default-light (default but adjusted for lighter backgrounds)
            - gruvbox       (a bright theme with 'retro groove' colors)
            - gruvbox-light (gruvbox but adjusted for lighter backgrounds)
            - nord          (an arctic, north-bluish color palette)
            - nord-light    (nord but adjusted for lighter backgrounds)"
        }
    )]
    pub theme: Option<String>,
}

/// Other arguments. This just handle options that are for help/version
/// displaying.
#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Other Options", rename_all = "snake_case")]
pub struct OtherArgs {
    #[arg(short = 'h', long, action = ArgAction::Help, help = "Prints help info (for more details use '--help'.")]
    help: (),

    #[arg(short = 'V', long, action = ArgAction::Version, help = "Prints version information.")]
    version: (),
}

/// Returns a [`BottomArgs`].
pub fn get_args() -> BottomArgs {
    BottomArgs::parse()
}

/// Returns an [`Command`] based off of [`BottomArgs`].
#[cfg(test)]
pub(crate) fn build_cmd() -> Command {
    BottomArgs::command()
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn verify_cli() {
        build_cmd().debug_assert();
    }

    #[test]
    fn no_default_help_heading() {
        let mut cmd = build_cmd();

        let help_str = cmd.render_help();
        assert!(
            !help_str.to_string().contains("\nOptions:\n"),
            "the default 'Options' heading should not exist; if it does then an argument is \
            missing a help heading."
        );

        let long_help_str = cmd.render_long_help();
        assert!(
            !long_help_str.to_string().contains("\nOptions:\n"),
            "the default 'Options' heading should not exist; if it does then an argument is \
            missing a help heading."
        );
    }

    #[test]
    fn catch_incorrect_long_args() {
        // Set this to allow certain ones through if needed.
        let allow_list: HashSet<&str> = vec![].into_iter().collect();
        let cmd = build_cmd();

        for opts in cmd.get_opts() {
            let long_flag = opts.get_long().unwrap();

            if !allow_list.contains(long_flag) {
                assert!(
                    long_flag.len() < 30,
                    "the long help arg '{long_flag}' might be set wrong, please take a look!"
                );
            }
        }
    }
}
