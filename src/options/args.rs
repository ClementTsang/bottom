//! Argument parsing via clap.
//!
//! Note that you probably want to keep this as a single file so the build script doesn't
//! trip all over itself.

// TODO: New sections are misaligned! See if we can get that fixed.

use std::{cmp::Ordering, path::PathBuf};

use clap::*;
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

pub fn get_matches() -> ArgMatches {
    build_app().get_matches()
}

/// Returns an [`Ordering`] for two [`Arg`] values.
///
/// Note this assumes that they both have a _long_ name, and will
/// panic if either are missing!
fn sort_args(a: &Arg, b: &Arg) -> Ordering {
    let a = a.get_long().unwrap();
    let b = b.get_long().unwrap();

    a.cmp(b)
}

/// Create an array of [`Arg`] values. If there is more than one value, then
/// they will be sorted by their long name. Note this sort will panic if
/// any [`Arg`] does not have a long name!
macro_rules! args {
    ( $arg:expr $(,)?) => {
        [$arg]
    };
    ( $( $arg:expr ),+ $(,)? ) => {
        {
            let mut args = [ $( $arg, )* ];
            args.sort_unstable_by(sort_args);
            args
        }
    };
}

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
    pub(crate) general: GeneralArgs,

    #[command(flatten)]
    pub(crate) process: ProcessArgs,

    #[command(flatten)]
    pub(crate) temperature: TemperatureArgs,

    #[command(flatten)]
    pub(crate) cpu: CpuArgs,

    #[command(flatten)]
    pub(crate) memory: MemoryArgs,

    #[command(flatten)]
    pub(crate) network: NetworkArgs,

    #[cfg(feature = "battery")]
    #[command(flatten)]
    pub(crate) battery: BatteryArgs,

    #[cfg(feature = "gpu")]
    #[command(flatten)]
    pub(crate) gpu: GpuArgs,

    #[command(flatten)]
    pub(crate) style: StyleArgs,

    #[command(flatten)]
    pub(crate) other: OtherArgs,
}

/// General arguments/config options.
#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "General Options", rename_all = "snake_case")]
pub(crate) struct GeneralArgs {
    #[arg(
        long,
        help = "Temporarily shows the time scale in graphs.",
        long = "Automatically hides the time scale in graphs after being shown for a brief moment when zoomed \
                in/out. If time is disabled via --hide_time then this will have no effect."
    )]
    pub(crate) autohide_time: Option<bool>,

    #[arg(
        short = 'b',
        long,
        help = "Hides graphs and uses a more basic look.",
        long_help = "Hides graphs and uses a more basic look, largely inspired by htop's design."
    )]
    pub(crate) basic: Option<bool>,

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
    pub(crate) config_location: Option<PathBuf>,

    #[arg(
        short = 't',
        long,
        value_name = "TIME",
        help = "Default time value for graphs.",
        long_help = "Default time value for graphs. Either a number in milliseconds or a 'human duration' \
                    (e.g. 60s, 10m). Defaults to 60s, must be at least 30s."
    )]
    pub(crate) default_time_value: Option<String>,

    // TODO: Charts are broken in the manpage
    #[arg(
        long,
        requires_all = ["default_widget_type"],
        value_name = "N",
        help = "Sets the n'th selected widget type as the default. Use --help for more info.",
        long_help = indoc! {
            "Sets the n'th selected widget type to use as the default widget.
            Requires 'default_widget_type' to also be set, and defaults to 1.

            This reads from left to right, top to bottom. For example, suppose
            we have a layout that looks like:
            +-------------------+-----------------------+
            |      CPU (1)      |        CPU (2)        |
            +---------+---------+-------------+---------+
            | Process | CPU (3) | Temperature | CPU (4) |
            +---------+---------+-------------+---------+

            And we set our default widget type to 'CPU'. If we set
            '--default_widget_count 1', then it would use the CPU (1) as
            the default widget. If we set '--default_widget_count 3', it would
            use CPU (3) as the default instead."
        }
    )]
    pub(crate) default_widget_count: Option<u32>,

    #[arg(
        long,
        value_name = "WIDGET",
        help = "Sets the default widget type. Use --help for more info.\n", // Newline to force the possible values to be on the next line.
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

            Setting '--default_widget_type Temp' will make the temperature widget selected by default."
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
    pub(crate) default_widget_type: Option<String>,

    #[arg(
        long,
        help = "Disables mouse clicks.",
        long_help = "Disables mouse clicks from interacting with bottom."
    )]
    pub(crate) disable_click: Option<bool>,

    #[arg(
        short = 'm',
        long,
        help = "Uses a dot marker for graphs.",
        long_help = "Uses a dot marker for graphs as opposed to the default braille marker."
    )]
    pub(crate) dot_marker: Option<bool>,

    #[arg(
        short = 'e',
        long,
        help = "Expand the default widget upon starting the app.",
        long_help = "Expand the default widget upon starting the app. This flag has no effect in basic mode (--basic)."
    )]
    pub(crate) expanded: Option<bool>,

    #[arg(long, help = "Hides spacing between table headers and entries.")]
    pub(crate) hide_table_gap: Option<bool>,

    #[arg(long, help = "Hides the time scale from being shown.")]
    pub(crate) hide_time: Option<bool>,

    #[arg(
        short = 'r',
        long,
        value_name = "TIME",
        help = "Sets how often data is refreshed.",
        long_help = "Sets how often data is refreshed. Either a number in milliseconds or a 'human duration' \
                    (e.g. 1s, 1m). Defaults to 1s, must be at least 250ms. Smaller values may result in \
                    higher system resource usage."
    )]
    pub(crate) rate: Option<String>,

    #[arg(
        long,
        value_name = "TIME",
        help = "How far back data will be stored up to.",
        long_help = "How far back data will be stored up to. Either a number in milliseconds or a 'human duration' \
                    (e.g. 10m, 1h). Defaults to 10 minutes, and must be at least  1 minute. Larger values \
                    may result in higher memory usage."
    )]
    pub(crate) retention: Option<String>,

    #[arg(long, help = "Show the current item entry position for table widgets.")]
    pub(crate) show_table_scroll_position: Option<bool>,

    #[arg(
        short = 'd',
        long,
        value_name = "TIME",
        help = "The amount of time changed upon zooming.",
        long_help = "How much time the x-axis shifts by each time you zoom in or out. Either a number in milliseconds \
                    or a 'human duration' (e.g. 15s, 1m). Defaults to 15 seconds."
    )]
    pub(crate) time_delta: Option<String>,
}

fn general_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("General Options");

    let autohide_time = Arg::new("autohide_time")
        .long("autohide_time")
        .action(ArgAction::SetTrue)
        .help("Temporarily shows the time scale in graphs.")
        .long_help(
            "Automatically hides the time scale in graphs after being shown for a brief moment when zoomed \
            in/out. If time is disabled via --hide_time then this will have no effect."
        );

    let basic = Arg::new("basic")
        .short('b')
        .long("basic")
        .action(ArgAction::SetTrue)
        .help("Hides graphs and uses a more basic look.")
        .long_help("Hides graphs and uses a more basic look, largely inspired by htop's design.");

    let config_location = Arg::new("config_location")
        .short('C')
        .long("config")
        .action(ArgAction::Set)
        .value_name("CONFIG PATH")
        .help("Sets the location of the config file.")
        .long_help(
            "Sets the location of the config file. Expects a config file in the TOML format. \
            If it doesn't exist, a default config file is created at the path. If no path is provided, \
            the default config location will be used."
        )
        .value_hint(ValueHint::AnyPath);

    let default_time_value = Arg::new("default_time_value")
        .short('t')
        .long("default_time_value")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("Default time value for graphs.")
        .long_help(
            "Default time value for graphs. Either a number in milliseconds or a 'human duration' \
            (e.g. 60s, 10m). Defaults to 60s, must be at least 30s.",
        );

    // TODO: Charts are broken in the manpage
    let default_widget_count = Arg::new("default_widget_count")
        .long("default_widget_count")
        .action(ArgAction::Set)
        .requires_all(["default_widget_type"])
        .value_name("N")
        .help("Sets the N'th selected widget type as the default.")
        .long_help(indoc! {
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
        });

    let default_widget_type = Arg::new("default_widget_type")
        .long("default_widget_type")
        .action(ArgAction::Set)
        .value_name("WIDGET")
        .help("Sets the default widget type, use `--help` for info.")
        .long_help(indoc!{
            "Sets which widget type to use as the default widget. For the default \
            layout, this defaults to the 'process' widget. For a custom layout, it defaults \
            to the first widget it sees.

            For example, suppose we have a layout that looks like:
            +-------------------+-----------------------+
            |      CPU (1)      |        CPU (2)        |
            +---------+---------+-------------+---------+
            | Process | CPU (3) | Temperature | CPU (4) |
            +---------+---------+-------------+---------+

            Setting '--default_widget_type temperature' will make the temperature widget selected by default."
        })
        .value_parser([
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
        ]);

    let disable_click = Arg::new("disable_click")
        .long("disable_click")
        .action(ArgAction::SetTrue)
        .help("Disables mouse clicks.")
        .long_help("Disables mouse clicks from interacting with bottom.");

    // TODO: Change this to accept a string with the type of marker.
    let dot_marker = Arg::new("dot_marker")
        .short('m')
        .long("dot_marker")
        .action(ArgAction::SetTrue)
        .help("Uses a dot marker for graphs.")
        .long_help("Uses a dot marker for graphs as opposed to the default braille marker.");

    let expanded = Arg::new("expanded")
        .short('e')
        .long("expanded")
        .action(ArgAction::SetTrue)
        .help("Expand the default widget upon starting the app.")
        .long_help("Expand the default widget upon starting the app. This flag has no effect in basic mode (--basic).");

    let hide_table_gap = Arg::new("hide_table_gap")
        .long("hide_table_gap")
        .action(ArgAction::SetTrue)
        .help("Hides spacing between table headers and entries.");

    let hide_time = Arg::new("hide_time")
        .long("hide_time")
        .action(ArgAction::SetTrue)
        .help("Hides the time scale from being shown.");

    let rate = Arg::new("rate")
        .short('r')
        .long("rate")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("Sets how often data is refreshed.")
        .long_help(
            "Sets how often data is refreshed. Either a number in milliseconds or a 'human duration' \
            (e.g. 1s, 1m). Defaults to 1s, must be at least 250ms. Smaller values may result in \
            higher system resource usage."
        );

    // TODO: Unify how we do defaults.
    let retention = Arg::new("retention")
        .long("retention")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("How far back data will be stored up to.")
        .long_help(
            "How far back data will be stored up to. Either a number in milliseconds or a 'human duration' \
            (e.g. 10m, 1h). Defaults to 10 minutes, and must be at least  1 minute. Larger values \
            may result in higher memory usage."
        );

    let show_table_scroll_position = Arg::new("show_table_scroll_position")
        .long("show_table_scroll_position")
        .action(ArgAction::SetTrue)
        .help("Shows the scroll position tracker in table widgets.")
        .long_help("Shows the list scroll position tracker in the widget title for table widgets.");

    let time_delta = Arg::new("time_delta")
        .short('d')
        .long("time_delta")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("The amount of time changed upon zooming.")
        .long_help(
            "The amount of time changed when zooming in/out. Takes a number in \
            milliseconds or a human duration (e.g. 30s). The minimum is 1s, and \
            defaults to 15s.",
        );

    cmd.args(args![
        autohide_time,
        basic,
        config_location,
        default_widget_count,
        default_time_value,
        default_widget_type,
        disable_click,
        dot_marker,
        expanded,
        hide_table_gap,
        hide_time,
        rate,
        retention,
        show_table_scroll_position,
        time_delta,
    ])
}

/// Process arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Process Options", rename_all = "snake_case")]
pub(crate) struct ProcessArgs {
    #[arg(
        short = 'S',
        long,
        help = "Enables case sensitivity by default.",
        long_help = "Enables case sensitivity by default when searching for a process."
    )]
    pub(crate) case_sensitive: Option<bool>,

    // TODO: Rename this.
    #[arg(
        short = 'u',
        long,
        help = "Calculates process CPU usage as a percentage of current usage rather than total usage."
    )]
    pub(crate) current_usage: Option<bool>,

    // TODO: Disable this on Windows?
    #[arg(
        long,
        help = "Hides additional stopping options Unix-like systems.",
        long_help = "Hides additional stopping options Unix-like systems. Signal 15 (TERM) will be sent when \
                    stopping a process."
    )]
    pub(crate) disable_advanced_kill: Option<bool>,

    #[arg(
        short = 'g',
        long,
        help = "Groups processes with the same name by default."
    )]
    pub(crate) group_processes: Option<bool>,

    #[arg(
        long,
        help = "Defaults to showing process memory usage by value.",
        long_help = "Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage."
    )]
    pub(crate) mem_as_value: Option<bool>,

    #[arg(
        long,
        help = "Shows the full command name instead of just the process name by default."
    )]
    pub(crate) process_command: Option<bool>,

    #[arg(short = 'R', long, help = "Enables regex by default while searching.")]
    pub(crate) regex: Option<bool>,

    #[arg(
        short = 'T',
        long,
        help = "Makes the process widget use tree mode by default."
    )]
    pub(crate) tree: Option<bool>,

    #[arg(
        short = 'n',
        long,
        help = "Show process CPU% usage without averaging over the number of CPU cores."
    )]
    pub(crate) unnormalized_cpu: Option<bool>,

    #[arg(
        short = 'W',
        long,
        help = "Enables whole-word matching by default while searching."
    )]
    pub(crate) whole_word: Option<bool>,
}

fn process_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Process Options");

    let case_sensitive = Arg::new("case_sensitive")
        .short('S')
        .long("case_sensitive")
        .action(ArgAction::SetTrue)
        .help("Enables case sensitivity by default.")
        .long_help("Enables case sensitivity by default when searching for a process.");

    // TODO: Rename this.
    let current_usage = Arg::new("current_usage")
        .short('u')
        .long("current_usage")
        .action(ArgAction::SetTrue)
        .help("Calculates process CPU usage as a percentage of current usage rather than total usage.");

    // TODO: Disable this on Windows?
    let disable_advanced_kill = Arg::new("disable_advanced_kill")
        .long("disable_advanced_kill")
        .action(ArgAction::SetTrue)
        .help("Hides additional stopping options Unix-like systems.")
        .long_help(
            "Hides additional stopping options Unix-like systems. Signal 15 (TERM) will be sent when \
            stopping a process.",
        );

    let group_processes = Arg::new("group_processes")
        .short('g')
        .long("group_processes")
        .action(ArgAction::SetTrue)
        .help("Groups processes with the same name by default.");

    let mem_as_value = Arg::new("mem_as_value")
        .long("mem_as_value")
        .action(ArgAction::SetTrue)
        .help("Defaults to showing process memory usage by value.")
        .long_help("Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage.");

    let process_command = Arg::new("process_command")
        .long("process_command")
        .action(ArgAction::SetTrue)
        .help("Shows the full command name instead of the process name by default.");

    let regex = Arg::new("regex")
        .short('R')
        .long("regex")
        .action(ArgAction::SetTrue)
        .help("Enables regex by default while searching.");

    let tree = Arg::new("tree")
        .short('T')
        .long("tree")
        .action(ArgAction::SetTrue)
        .help("Makes the process widget use tree mode by default.");

    let unnormalized_cpu = Arg::new("unnormalized_cpu")
        .short('n')
        .long("unnormalized_cpu")
        .action(ArgAction::SetTrue)
        .help("Show process CPU% usage without averaging over the number of CPU cores.");

    let whole_word = Arg::new("whole_word")
        .short('W')
        .long("whole_word")
        .action(ArgAction::SetTrue)
        .help("Enables whole-word matching by default while searching.");

    let args = args![
        case_sensitive,
        current_usage,
        disable_advanced_kill,
        group_processes,
        mem_as_value,
        process_command,
        regex,
        tree,
        unnormalized_cpu,
        whole_word,
    ];

    cmd.args(args)
}

/// Temperature arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Temperature Options", rename_all = "snake_case")]
#[group(id = "temperature_unit", multiple = false)]
pub(crate) struct TemperatureArgs {
    #[arg(
        short = 'c',
        long,
        group = "temperature_unit",
        help = "Use Celsius as the temperature unit. Default.",
        long_help = "Use Celsius as the temperature unit. This is the default option."
    )]
    pub(crate) celsius: bool,

    #[arg(
        short = 'f',
        long,
        group = "temperature_unit",
        help = "Use Fahrenheit as the temperature unit."
    )]
    pub(crate) fahrenheit: bool,

    #[arg(
        short = 'k',
        long,
        group = "temperature_unit",
        help = "Use Kelvin as the temperature unit."
    )]
    pub(crate) kelvin: bool,
}

fn temperature_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Temperature Options");

    let celsius = Arg::new("celsius")
        .short('c')
        .long("celsius")
        .action(ArgAction::SetTrue)
        .help("Use Celsius as the temperature unit. Default.")
        .long_help("Use Celsius as the temperature unit. This is the default option.");

    let fahrenheit = Arg::new("fahrenheit")
        .short('f')
        .long("fahrenheit")
        .action(ArgAction::SetTrue)
        .help("Use Fahrenheit as the temperature unit.");

    let kelvin = Arg::new("kelvin")
        .short('k')
        .long("kelvin")
        .action(ArgAction::SetTrue)
        .help("Use Kelvin as the temperature unit.");

    let temperature_group = ArgGroup::new("TEMPERATURE_TYPE").args([
        celsius.get_id(),
        fahrenheit.get_id(),
        kelvin.get_id(),
    ]);

    cmd.args(args![celsius, fahrenheit, kelvin])
        .group(temperature_group)
}

/// The default selection of the CPU widget. If the given selection is invalid,
/// we will fall back to all.
#[derive(Clone, Copy, Debug, Default)]
pub enum CpuDefault {
    #[default]
    All,
    Average,
}

impl From<&str> for CpuDefault {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "all" => CpuDefault::All,
            "avg" | "average" => CpuDefault::Average,
            _ => CpuDefault::All,
        }
    }
}

/// CPU arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "CPU Options", rename_all = "snake_case")]
pub(crate) struct CpuArgs {
    #[arg(
        long,
        help = "Sets which CPU entry is selected by default.",
        value_name = "ENTRY",
        value_parser = ["all", "avg"],
        default_value = "all"
    )]
    pub(crate) default_cpu_entry: CpuDefault,

    #[arg(short = 'a', long, help = "Hides the average CPU usage entry.")]
    pub(crate) hide_avg_cpu: Option<bool>,

    // TODO: Maybe rename this or fix this? Should this apply to all "left legends"?
    #[arg(
        short = 'l',
        long,
        help = "Puts the CPU chart legend on the left side."
    )]
    pub(crate) left_legend: Option<bool>,
}

fn cpu_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("CPU Options");

    // let default_cpu_entry = Arg::new("");

    let hide_avg_cpu = Arg::new("hide_avg_cpu")
        .short('a')
        .long("hide_avg_cpu")
        .action(ArgAction::SetTrue)
        .help("Hides the average CPU usage entry.");

    let cpu_left_legend = Arg::new("cpu_left_legend")
        .long("cpu_left_legend")
        .action(ArgAction::SetTrue)
        .help("Puts the CPU chart legend on the left side.");

    cmd.args(args![hide_avg_cpu, cpu_left_legend])
}

/// Memory argument/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Memory Options", rename_all = "snake_case")]
pub(crate) struct MemoryArgs {
    #[cfg(not(target_os = "windows"))]
    #[arg(
        long,
        help = "Enables collecting and displaying cache and buffer memory."
    )]
    pub(crate) enable_cache_memory: Option<bool>,
}

fn mem_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Memory Options");

    let memory_legend = Arg::new("memory_legend")
        .long("memory_legend")
        .action(ArgAction::Set)
        .value_name("POSITION")
        .ignore_case(true)
        .help("Where to place the legend for the memory chart widget.")
        .value_parser(CHART_WIDGET_POSITIONS);

    #[cfg(not(target_os = "windows"))]
    {
        let enable_cache_memory = Arg::new("enable_cache_memory")
            .long("enable_cache_memory")
            .action(ArgAction::SetTrue)
            .help("Enable collecting and displaying cache and buffer memory.");

        cmd.args(args![enable_cache_memory, memory_legend])
    }
    #[cfg(target_os = "windows")]
    {
        cmd.arg(memory_legend)
    }
}

/// Network arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Network Options", rename_all = "snake_case")]
pub(crate) struct NetworkArgs {
    // TODO: Rename some of these to remove the network prefix for serde.
    #[arg(
        long,
        help = "Displays the network widget using bytes.",
        long_help = "Displays the network widget using bytes. Defaults to bits."
    )]
    pub(crate) network_use_bytes: Option<bool>,

    #[arg(
        long,
        help = "Displays the network widget with binary prefixes.",
        long_help = "Displays the network widget with binary prefixes (e.g. kibibits, mebibits) rather than a decimal \
                    prefixes (e.g. kilobits, megabits). Defaults to decimal prefixes."
    )]
    pub(crate) network_use_binary_prefix: Option<bool>,

    #[arg(
        long,
        help = "Displays the network widget with a log scale.",
        long_help = "Displays the network widget with a log scale. Defaults to a non-log scale."
    )]
    pub(crate) network_use_log: Option<bool>,

    #[arg(
        long,
        help = "(DEPRECATED) Uses a separate network legend.",
        long_help = "(DEPRECATED) Uses separate network widget legend. This display is not tested and may be broken."
    )]
    pub(crate) use_old_network_legend: Option<bool>,
}

fn network_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Network Options");

    let network_legend = Arg::new("network_legend")
        .long("network_legend")
        .action(ArgAction::Set)
        .value_name("POSITION")
        .ignore_case(true)
        .help("Where to place the legend for the network chart widget.")
        .value_parser(CHART_WIDGET_POSITIONS);

    let network_use_bytes = Arg::new("network_use_bytes")
        .long("network_use_bytes")
        .action(ArgAction::SetTrue)
        .help("Displays the network widget using bytes.")
        .long_help("Displays the network widget using bytes. Defaults to bits.");

    let network_use_binary_prefix = Arg::new("network_use_binary_prefix")
        .long("network_use_binary_prefix")
        .action(ArgAction::SetTrue)
        .help("Displays the network widget with binary prefixes.")
        .long_help(
            "Displays the network widget with binary prefixes (e.g. kibibits, mebibits) rather than a decimal \
            prefixes (e.g. kilobits, megabits). Defaults to decimal prefixes."
        );

    let network_use_log = Arg::new("network_use_log")
        .long("network_use_log")
        .action(ArgAction::SetTrue)
        .help("Displays the network widget with a log scale.")
        .long_help("Displays the network widget with a log scale. Defaults to a non-log scale.");

    // TODO: Change this to be configured as network graph type?
    let use_old_network_legend = Arg::new("use_old_network_legend")
        .long("use_old_network_legend")
        .action(ArgAction::SetTrue)
        .help("(DEPRECATED) Uses a separated network legend.")
        .long_help("(DEPRECATED) Uses separated network widget legend. This display is not tested and may be broken.");

    cmd.args(args![
        network_legend,
        network_use_bytes,
        network_use_log,
        network_use_binary_prefix,
        use_old_network_legend,
    ])
}

/// Battery arguments/config options.
#[cfg(feature = "battery")]
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Battery Options", rename_all = "snake_case")]
pub(crate) struct BatteryArgs {
    #[arg(
        long,
        help = "Shows the battery widget in default/basic mode.",
        long_help = "Shows the battery widget in default or basic mode, if there is as battery available. This \
                    has no effect on custom layouts; if the battery widget is desired for a custom layout, explicitly \
                    specify it."
    )]
    pub(crate) battery: Option<bool>,
}

#[cfg(feature = "battery")]
fn battery_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Battery Options");

    let battery = Arg::new("battery")
        .long("battery")
        .action(ArgAction::SetTrue)
        .help("Shows the battery widget in non-custom layouts.")
        .long_help(
            "Shows the battery widget in default or basic mode, if there is as battery available. This \
            has no effect on custom layouts; if the battery widget is desired for a custom layout, explicitly \
            specify it."
        );

    cmd.arg(battery)
}

/// GPU arguments/config options.
#[cfg(feature = "gpu")]
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "GPU Options", rename_all = "snake_case")]
pub(crate) struct GpuArgs {
    #[arg(long, help = "Enables collecting and displaying GPU usage.")]
    pub(crate) enable_gpu: Option<bool>,
}

#[cfg(feature = "gpu")]
fn gpu_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("GPU Options");

    let enable_gpu = Arg::new("enable_gpu")
        .long("enable_gpu")
        .action(ArgAction::SetTrue)
        .help("Enable collecting and displaying GPU usage.");

    cmd.arg(enable_gpu)
}

/// Style arguments/config options.
#[derive(Args, Clone, Debug, Default)]
#[command(next_help_heading = "Style Options", rename_all = "snake_case")]
pub(crate) struct StyleArgs {
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
            "custom",

        ],
        hide_possible_values = true,
        help = "Use a color scheme, use --help for info on the colors.\n
                [possible values: default, default-light, gruvbox, gruvbox-light, nord, nord-light]",
        long_help = indoc! {
            "Use a pre-defined color scheme. Currently supported values are:
            - default
            - default-light (default but adjusted for lighter backgrounds)
            - gruvbox       (a bright theme with 'retro groove' colors)
            - gruvbox-light (gruvbox but adjusted for lighter backgrounds)
            - nord          (an arctic, north-bluish color palette)
            - nord-light    (nord but adjusted for lighter backgrounds)
            - custom        (use a custom color scheme defined in the config file)"
        }
    )]
    pub(crate) color: Option<String>,
}

fn style_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Style Options");

    // TODO: File an issue with manpage, it cannot render charts correctly.
    let color = Arg::new("color")
        .long("color")
        .action(ArgAction::Set)
        .value_name("SCHEME")
        .value_parser([
            "default",
            "default-light",
            "gruvbox",
            "gruvbox-light",
            "nord",
            "nord-light",
        ])
        .hide_possible_values(true)
        .help(indoc! {
            "Use a color scheme, use `--help` for info on the colors. [possible values: default, default-light, gruvbox, gruvbox-light, nord, nord-light]",
        })
        .long_help(indoc! {
            "Use a pre-defined color scheme. Currently supported values are:
            - default
            - default-light (default but adjusted for lighter backgrounds)
            - gruvbox       (a bright theme with 'retro groove' colors)
            - gruvbox-light (gruvbox but adjusted for lighter backgrounds)
            - nord          (an arctic, north-bluish color palette)
            - nord-light    (nord but adjusted for lighter backgrounds)"
        });

    cmd.arg(color)
}

/// Other arguments. This just handle options that are for help/version displaying.
#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Other Options", rename_all = "snake_case")]
pub(crate) struct OtherArgs {
    #[arg(short = 'h', long, action = ArgAction::Help, help = "Prints help info (for more details use `--help`.")]
    help: (),

    #[arg(short = 'v', long, action = ArgAction::Version, help = "Prints version information.")]
    version: (),
}

fn other_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Other Options");

    let help = Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help("Prints help info (for more details use `--help`.");

    let version = Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::Version)
        .help("Prints version information.");

    cmd.args([help, version])
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

pub fn build_app() -> Command {
    let cmd = Command::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .disable_help_flag(true)
        .disable_version_flag(true)
        .color(ColorChoice::Auto)
        .help_template(TEMPLATE)
        .override_usage(USAGE)
        .version(VERSION);

    [
        general_args,
        process_args,
        temperature_args,
        cpu_args,
        mem_args,
        network_args,
        #[cfg(feature = "battery")]
        battery_args,
        #[cfg(feature = "gpu")]
        gpu_args,
        style_args,
        other_args,
    ]
    .into_iter()
    .fold(cmd, |c, f| f(c))
}

#[cfg(test)]
mod test {
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
    }
}
