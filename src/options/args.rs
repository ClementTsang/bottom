//! Argument parsing via clap.
//!
//! Note that you probably want to keep this as a single file so the build script doesn't
//! trip all over itself.

// TODO: New sections are misaligned! See if we can get that fixed.

use std::cmp::Ordering;

use clap::*;
use indoc::indoc;

/// Returns an [`Ordering`] for two [`Arg`] values.
///
/// Note this assumes that _both have a long_ name, and will
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

/// The arguments for bottom.
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
pub(crate) struct Args {
    #[command(flatten)]
    pub(crate) general_args: GeneralArgs,

    #[command(flatten)]
    pub(crate) process_args: ProcessArgs,

    #[command(flatten)]
    pub(crate) temperature_args: TemperatureArgs,

    #[command(flatten)]
    pub(crate) cpu_args: CpuArgs,

    #[command(flatten)]
    pub(crate) mem_args: MemArgs,

    #[command(flatten)]
    pub(crate) network_args: NetworkArgs,

    #[cfg(feature = "battery")]
    #[command(flatten)]
    pub(crate) battery_args: BatteryArgs,

    #[cfg(feature = "gpu")]
    #[command(flatten)]
    pub(crate) gpu_args: GpuArgs,

    #[command(flatten)]
    pub(crate) style_args: StyleArgs,

    #[command(flatten)]
    pub(crate) other_args: OtherArgs,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "General Options")]
pub(crate) struct GeneralArgs {
    #[arg(
        long,
        help = "Temporarily shows the time scale in graphs.",
        long_help = "Automatically hides the time scale in graphs after being shown for a brief moment when zoomed \
                    in/out. If time is disabled via --hide_time then this will have no effect."
    )]
    pub(crate) autohide_time: bool,

    #[arg(
        short = 'b',
        long,
        help = "Hides graphs and uses a more basic look.",
        long_help = "Hides graphs and uses a more basic look. Design is largely inspired by htop's."
    )]
    pub(crate) basic: bool,

    #[arg(
        short = 'C',
        long,
        value_name = "PATH",
        help = "Sets the location of the config file.",
        long_help = "Sets the location of the config file. Expects a config file in the TOML format. \
                    If it doesn't exist, a default config file is created at the path."
    )]
    pub(crate) config_location: String,

    #[arg(
        short = 't',
        long,
        value_name = "TIME",
        help = "Default time value for graphs.",
        long_help = "The default time value for graphs. Takes a number in milliseconds or a human \
                    duration (e.g. 60s). The minimum time is 30s, and the default is 60s."
    )]
    pub(crate) default_time_value: String,

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
    pub(crate) default_widget_count: u32,

    #[arg(
        long,
        value_name = "WIDGET",
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
            #[cfg(not(feature = "battery"))]
            "batt",
            #[cfg(not(feature = "battery"))]
            "battery",
        ],
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

            Setting '--default_widget_type Temp' will make the temperature widget selected by default."
        }
    )]
    pub(crate) default_widget_type: String,

    #[arg(
        long,
        help = "Disables mouse clicks.",
        long_help = "Disables mouse clicks from interacting with bottom."
    )]
    pub(crate) disable_click: bool,

    #[arg(
        short = 'm',
        long,
        help = "Uses a dot marker for graphs.",
        long_help = "Uses a dot marker for graphs as opposed to the default braille marker."
    )]
    pub(crate) dot_marker: bool,

    #[arg(
        short = 'e',
        long,
        help = "Expand the default widget upon starting the app.",
        long_help = "Expand the default widget upon starting the app. This flag has no effect in basic mode (--basic)."
    )]
    pub(crate) expanded: bool,

    #[arg(long, help = "Hides spacing between table headers and entries.")]
    pub(crate) hide_table_gap: bool,

    #[arg(
        long,
        help = "Hides the time scale.",
        long_help = "Completely hides the time scale from being shown."
    )]
    pub(crate) hide_time: bool,

    #[arg(
        short = 'r',
        long,
        value_name = "TIME",
        help = "Sets how often data is refreshed.",
        long_help = "Sets how often data is refreshed. Takes a number in milliseconds or a human-readable duration \
                    (e.g. 5s). The minimum is 250ms, and defaults to 1000ms. Smaller values may result in higher \
                    system usage by bottom."
    )]
    pub(crate) rate: String,

    #[arg(
        long,
        value_name = "TIME",
        help = "The timespan of data stored.",
        long_help = "How much data is stored at once in terms of time. Takes a number in milliseconds or a \
                    human-readable duration (e.g. 20m), with a minimum of 1 minute. Note that higher values \
                    will take up more memory. Defaults to 10 minutes."
    )]
    pub(crate) retention: String,

    #[arg(
        long,
        help = "Shows the scroll position tracker in table widgets.",
        long_help = "Shows the list scroll position tracker in the widget title for table widgets."
    )]
    pub(crate) show_table_scroll_position: bool,

    #[arg(
        short = 'd',
        long,
        value_name = "TIME",
        help = "The amount of time changed upon zooming.",
        long_help = "The amount of time changed when zooming in/out. Takes a number in milliseconds or a \
                    human-readable duration (e.g. 30s). The minimum is 1s, and defaults to 15s."
    )]
    pub(crate) time_delta: String,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Process Options")]
pub(crate) struct ProcessArgs {
    #[arg(
        short = 'S',
        long,
        help = "Enables case sensitivity by default.",
        long_help = "When searching for a process, enables case sensitivity by default."
    )]
    pub(crate) case_sensitive: bool,

    // TODO: Rename this.
    #[arg(
        short = 'u',
        long,
        help = "Sets process CPU% to be based on current CPU%.",
        long_help = "Sets process CPU% usage to be based on the current system CPU% usage rather than total CPU usage."
    )]
    pub(crate) current_usage: bool,

    // TODO: Disable this on Windows?
    #[arg(
        long,
        help = "Hides advanced process killing options.",
        long_help = "Hides advanced options to stop a process on Unix-like systems. The only \
                    option shown is 15 (TERM)."
    )]
    pub(crate) disable_advanced_kill: bool,

    #[arg(
        short = 'g',
        long,
        help = "Groups processes with the same name by default."
    )]
    pub(crate) group_processes: bool,

    #[arg(long, help = "Show processes as their commands by default.")]
    pub(crate) process_command: bool,

    #[arg(short = 'R', long, help = "Enables regex by default while searching.")]
    pub(crate) regex: bool,

    #[arg(
        short = 'T',
        long,
        help = "Defaults the process widget be in tree mode."
    )]
    pub(crate) tree: bool,

    #[arg(
        short = 'n',
        long,
        help = "Show process CPU% usage without normalizing over the number of cores.",
        long_help = "Shows all process CPU% usage without averaging over the number of CPU cores in the system."
    )]
    pub(crate) unnormalized_cpu: bool,

    #[arg(
        short = 'W',
        long,
        help = "Enables whole-word matching by default while searching."
    )]
    pub(crate) whole_word: bool,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Temperature Options")]
#[group(multiple = false)]
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
        help = "Use Fahrenheit as the temperature unit. Default."
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

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "CPU Options")]
pub(crate) struct CpuArgs {
    #[arg(
        short = 'a',
        long,
        help = "Hides the average CPU usage entry.",
        long = "Hides the average CPU usage entry from being shown."
    )]
    pub(crate) hide_avg_cpu: bool,

    // TODO: Maybe rename this or fix this? Should this apply to all "left legends"?
    #[arg(
        short = 'l',
        long,
        help = "Puts the CPU chart legend to the left side.",
        long_help = "Puts the CPU chart legend to the left side rather than the right side."
    )]
    pub(crate) left_legend: bool,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Memory Options")]
pub(crate) struct MemArgs {
    #[cfg(not(target_os = "windows"))]
    #[arg(
        long,
        help = "Enables collecting and displaying cache and buffer memory."
    )]
    pub(crate) enable_cache_memory: bool,

    #[arg(
        long,
        help = "Defaults to showing process memory usage by value.",
        long_help = "Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage."
    )]
    pub(crate) mem_as_value: bool,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Network Options")]
pub(crate) struct NetworkArgs {
    #[arg(
        long,
        help = "Displays the network widget using bytes.",
        long_help = "Displays the network widget using bytes. Defaults to bits."
    )]
    pub(crate) network_use_bytes: bool,

    #[arg(
        long,
        help = "Displays the network widget with binary prefixes.",
        long_help = "Displays the network widget with binary prefixes (e.g. kibibits, mebibits) rather than a decimal \
                    prefixes (e.g. kilobits, megabits). Defaults to decimal prefixes."
    )]
    pub(crate) network_use_binary_prefix: bool,

    #[arg(
        long,
        help = "Displays the network widget with a log scale.",
        long_help = "Displays the network widget with a log scale. Defaults to a non-log scale."
    )]
    pub(crate) network_use_log: bool,

    #[arg(
        long,
        help = "(DEPRECATED) Uses a separate network legend.",
        long_help = "(DEPRECATED) Uses separate network widget legend. This display is not tested and may be broken."
    )]
    pub(crate) use_old_network_legend: bool,
}

#[cfg(feature = "battery")]
#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Battery Options")]
pub(crate) struct BatteryArgs {
    #[arg(
        long,
        help = "Shows the battery widget in default/basic mode.",
        long_help = "Shows the battery widget in default or basic mode, if there is as battery available. This \
                    has no effect on custom layouts; if the battery widget is desired for a custom layout, explicitly \
                    specify it."
    )]
    pub(crate) battery: bool,
}

#[cfg(feature = "gpu")]
#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "GPU Options")]
pub(crate) struct GpuArgs {
    #[arg(long, help = "Enables collecting and displaying GPU usage.")]
    pub(crate) enable_gpu: bool,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Style Options")]
pub(crate) struct StyleArgs {
    #[arg(
        long,
        value_name="SCHEME",
        value_parser=[
            "default",
            "default-light",
            "gruvbox",
            "gruvbox-light",
            "nord",
            "nord-light",

        ],
        hide_possible_values=true,
        help = "Use a color scheme, use --help for info on the colors. \
                [possible values: default, default-light, gruvbox, gruvbox-light, nord, nord-light]",
        long_help=indoc! {
            "Use a pre-defined color scheme. Currently supported values are:
            - default
            - default-light (default but adjusted for lighter backgrounds)
            - gruvbox       (a bright theme with 'retro groove' colors)
            - gruvbox-light (gruvbox but adjusted for lighter backgrounds)
            - nord          (an arctic, north-bluish color palette)
            - nord-light    (nord but adjusted for lighter backgrounds)"
        }
    )]
    pub(crate) color: String,
}

#[derive(Args, Clone, Debug)]
#[command(next_help_heading = "Other Options")]
pub(crate) struct OtherArgs {
    #[arg(short='h', long, action=ArgAction::Help, help="Prints help info (for more details use `--help`.")]
    help: (),

    #[arg(short='v', long, action=ArgAction::Version, help="Prints version information.")]
    version: (),
}

fn general_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("General Options");

    let autohide_time = Arg::new("autohide_time")
        .long("autohide_time")
        .action(ArgAction::SetTrue)
        .help("Temporarily shows the time scale in graphs.")
        .long_help(
            "Automatically hides the time scale in graphs after being shown for \
            a brief moment when zoomed in/out. If time is disabled via --hide_time \
            then this will have no effect.",
        );

    let basic = Arg::new("basic")
        .short('b')
        .long("basic")
        .action(ArgAction::SetTrue)
        .help("Hides graphs and uses a more basic look.")
        .long_help(
            "Hides graphs and uses a more basic look. Design is largely inspired by htop's.",
        );

    let disable_click = Arg::new("disable_click")
        .long("disable_click")
        .action(ArgAction::SetTrue)
        .help("Disables mouse clicks.")
        .long_help("Disables mouse clicks from interacting with the program.");

    let dot_marker = Arg::new("dot_marker")
        .short('m')
        .long("dot_marker")
        .action(ArgAction::SetTrue)
        .help("Uses a dot marker for graphs.")
        .long_help("Uses a dot marker for graphs as opposed to the default braille marker.");

    let hide_table_gap = Arg::new("hide_table_gap")
        .long("hide_table_gap")
        .action(ArgAction::SetTrue)
        .help("Hides spacing between table headers and entries.")
        .long_help("Hides the spacing between table headers and entries.");

    let hide_time = Arg::new("hide_time")
        .long("hide_time")
        .action(ArgAction::SetTrue)
        .help("Hides the time scale.")
        .long_help("Completely hides the time scale from being shown.");

    let left_legend = Arg::new("left_legend")
        .short('l')
        .long("left_legend")
        .action(ArgAction::SetTrue)
        .help("Puts the CPU chart legend to the left side.")
        .long_help("Puts the CPU chart legend to the left side rather than the right side.");

    let show_table_scroll_position = Arg::new("show_table_scroll_position")
        .long("show_table_scroll_position")
        .action(ArgAction::SetTrue)
        .help("Shows the scroll position tracker in table widgets.")
        .long_help("Shows the list scroll position tracker in the widget title for table widgets.");

    let config_location = Arg::new("config_location")
        .short('C')
        .long("config")
        .action(ArgAction::Set)
        .value_name("CONFIG PATH")
        .help("Sets the location of the config file.")
        .long_help(
            "Sets the location of the config file. Expects a config file in the TOML format.\
            If it doesn't exist, one is created.",
        )
        .value_hint(ValueHint::AnyPath);

    let default_time_value = Arg::new("default_time_value")
        .short('t')
        .long("default_time_value")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("Default time value for graphs.")
        .long_help(
            "Default time value for graphs. Takes a number in milliseconds or a human \
            duration (e.g. 60s). The minimum time is 30s, and the default is 60s.",
        );

    // TODO: Charts are broken in the manpage
    let default_widget_count = Arg::new("default_widget_count")
        .long("default_widget_count")
        .action(ArgAction::Set)
        .requires_all(["default_widget_type"])
        .value_name("INT")
        .help("Sets the n'th selected widget type as the default.")
        .long_help(indoc! {
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
        });

    let default_widget_type = Arg::new("default_widget_type")
        .long("default_widget_type")
        .action(ArgAction::Set)
        .value_name("WIDGET TYPE")
        .help("Sets the default widget type, use --help for info.")
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

            Setting '--default_widget_type Temp' will make the temperature widget selected by default."
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
            #[cfg(not(feature = "battery"))]
            "batt",
            #[cfg(not(feature = "battery"))]
            "battery",
        ]);

    let expanded_on_startup = Arg::new("expanded_on_startup")
        .short('e')
        .long("expanded")
        .action(ArgAction::SetTrue)
        .help("Expand the default widget upon starting the app.")
        .long_help(
            "Expand the default widget upon starting the app. \
            Same as pressing \"e\" inside the app. Use with \"default_widget_type\" \
            and \"default_widget_count\" to select the desired expanded widget. This \
            flag has no effect in basic mode (--basic).",
        );

    let rate = Arg::new("rate")
        .short('r')
        .long("rate")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("Sets the data refresh rate.")
        .long_help(
            "Sets the data refresh rate. Takes a number in milliseconds or a human\
            duration (e.g. 5s). The minimum is 250ms, and defaults to 1000ms. Smaller \
            values may take more computer resources.",
        );

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

    // TODO: Unify how we do defaults.
    let retention = Arg::new("retention")
        .long("retention")
        .action(ArgAction::Set)
        .value_name("TIME")
        .help("The timespan of data stored.")
        .long_help(
            "How much data is stored at once in terms of time. Takes a number \
            in milliseconds or a human duration (e.g. 20m), with a minimum of 1 minute. \
            Note that higher values will take up more memory. Defaults to 10 minutes.",
        );

    cmd.args(args![
        autohide_time,
        basic,
        disable_click,
        dot_marker,
        hide_table_gap,
        hide_time,
        left_legend,
        show_table_scroll_position,
        config_location,
        default_time_value,
        default_widget_count,
        default_widget_type,
        expanded_on_startup,
        rate,
        time_delta,
        retention,
    ])
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
        .help(
            "Use a color scheme, use --help for info on the colors. \
            [possible values: default, default-light, gruvbox, gruvbox-light, nord, nord-light]",
        )
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

fn temperature_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Temperature Options");

    let celsius = Arg::new("celsius")
        .short('c')
        .long("celsius")
        .action(ArgAction::SetTrue)
        .help("Use Celsius as the temperature unit.")
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

fn process_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Process Options");

    let case_sensitive = Arg::new("case_sensitive")
        .short('S')
        .long("case_sensitive")
        .action(ArgAction::SetTrue)
        .help("Enables case sensitivity by default.")
        .long_help("When searching for a process, enables case sensitivity by default.");

    let current_usage = Arg::new("current_usage")
        .short('u')
        .long("current_usage")
        .action(ArgAction::SetTrue)
        .help("Sets process CPU% to be based on current CPU%.")
        .long_help(
            "Sets process CPU% usage to be based on the current system CPU% usage rather \
            than total CPU usage.",
        );

    let unnormalized_cpu = Arg::new("unnormalized_cpu")
        .short('n')
        .long("unnormalized_cpu")
        .action(ArgAction::SetTrue)
        .help("Show process CPU% usage without normalizing over the number of cores.")
        .long_help(
            "Shows all process CPU% usage without averaging over the number of CPU cores \
            in the system.",
        );

    let group_processes = Arg::new("group_processes")
        .short('g')
        .long("group_processes")
        .action(ArgAction::SetTrue)
        .help("Groups processes with the same name by default.")
        .long_help("Groups processes with the same name by default.");

    let process_command = Arg::new("process_command")
        .long("process_command")
        .action(ArgAction::SetTrue)
        .help("Show processes as their commands by default.")
        .long_help("Show processes as their commands by default in the process widget.");

    let regex = Arg::new("regex")
        .short('R')
        .long("regex")
        .action(ArgAction::SetTrue)
        .help("Enables regex by default.")
        .long_help("When searching for a process, enables regex by default.");

    let disable_advanced_kill = Arg::new("disable_advanced_kill")
        .long("disable_advanced_kill")
        .action(ArgAction::SetTrue)
        .help("Hides advanced process killing.")
        .long_help(
            "Hides advanced options to stop a process on Unix-like systems. The only \
            option shown is 15 (TERM).",
        );

    let whole_word = Arg::new("whole_word")
        .short('W')
        .long("whole_word")
        .action(ArgAction::SetTrue)
        .help("Enables whole-word matching by default.")
        .long_help(
            "When searching for a process, return results that match the entire query by default.",
        );

    let tree = Arg::new("tree")
        .short('T')
        .long("tree")
        .action(ArgAction::SetTrue)
        .help("Defaults the process widget be in tree mode.")
        .long_help("Defaults to showing the process widget in tree mode.");

    let args = args![
        case_sensitive,
        current_usage,
        unnormalized_cpu,
        group_processes,
        process_command,
        regex,
        whole_word,
        disable_advanced_kill,
        tree,
    ];

    cmd.args(args)
}

fn cpu_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("CPU Options");

    let hide_avg_cpu = Arg::new("hide_avg_cpu")
        .short('a')
        .long("hide_avg_cpu")
        .action(ArgAction::SetTrue)
        .help("Hides the average CPU usage.")
        .long_help("Hides the average CPU usage from being shown.");

    // let default_avg_cpu = Arg::new("");

    cmd.args(args![hide_avg_cpu])
}

fn mem_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Memory Options");

    let mem_as_value = Arg::new("mem_as_value")
        .long("mem_as_value")
        .action(ArgAction::SetTrue)
        .help("Defaults to showing process memory usage by value.")
        .long_help(
            "Defaults to showing process memory usage by value. Otherwise, it defaults \
            to showing it by percentage.",
        );

    #[cfg(not(target_os = "windows"))]
    {
        let enable_cache_memory = Arg::new("enable_cache_memory")
            .long("enable_cache_memory")
            .action(ArgAction::SetTrue)
            .help("Enable collecting and displaying cache and buffer memory.");

        cmd.args(args![mem_as_value, enable_cache_memory])
    }
    #[cfg(target_os = "windows")]
    {
        cmd.arg(mem_as_value)
    }
}

fn network_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Network Options");

    // TODO: Change this to be configured as network graph type?
    let use_old_network_legend = Arg::new("use_old_network_legend")
        .long("use_old_network_legend")
        .action(ArgAction::SetTrue)
        .help("DEPRECATED - uses a separate network legend.")
        .long_help(
            "DEPRECATED - uses an older (pre-0.4), separate network widget legend. This \
            display is not tested anymore and may be broken.",
        );

    let network_use_bytes = Arg::new("network_use_bytes")
        .long("network_use_bytes")
        .action(ArgAction::SetTrue)
        .help("Displays the network widget using bytes.")
        .long_help("Displays the network widget using bytes. Defaults to bits.");

    let network_use_log = Arg::new("network_use_log")
        .long("network_use_log")
        .action(ArgAction::SetTrue)
        .help("Displays the network widget with a log scale.")
        .long_help("Displays the network widget with a log scale. Defaults to a non-log scale.");

    let network_use_binary_prefix = Arg::new("network_use_binary_prefix")
        .long("network_use_binary_prefix")
        .action(ArgAction::SetTrue)
        .help("Displays the network widget with binary prefixes.")
        .long_help(
            "Displays the network widget with binary prefixes (i.e. kibibits, mebibits) \
            rather than a decimal prefix (i.e. kilobits, megabits). Defaults to decimal prefixes.",
        );

    cmd.args(args![
        use_old_network_legend,
        network_use_bytes,
        network_use_log,
        network_use_binary_prefix,
    ])
}

#[cfg(feature = "battery")]
fn battery_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Battery Options");

    let battery = Arg::new("battery")
        .long("battery")
        .action(ArgAction::SetTrue)
        .help("Shows the battery widget.")
        .long_help(
            "Shows the battery widget in default or basic mode. No effect on custom layouts.",
        );

    cmd.arg(battery)
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

fn other_args(cmd: Command) -> Command {
    let cmd = cmd.next_help_heading("Other Options");

    let help = Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help("Prints help (see more info with '--help').");

    let version = Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::Version)
        .help("Prints version information.");

    cmd.args([help, version])
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

/// Returns an [`Args`].
pub fn new_build_app() -> Args {
    Args::parse()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        build_app().debug_assert();
    }

    #[test]
    fn no_default_help_heading() {
        let mut app = build_app();
        let help_str = app.render_help();

        assert!(
            !help_str.to_string().contains("\nOptions:\n"),
            "the default 'Options' heading should not exist; if it does then an argument is \
            missing a help heading."
        );
    }
}
