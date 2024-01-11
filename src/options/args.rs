//! Argument parsing via clap.
//!
//! Note that you probably want to keep this as a single file so the build script doesn't
//! trip all over itself.

use clap::{builder::PossibleValuesParser, *};

const DEFAULT_WIDGET_TYPE_STR: &str = {
    #[cfg(feature = "battery")]
    {
        "\
Sets which widget type to use as the default widget.
For the default layout, this defaults to the 'process' widget.
For a custom layout, it defaults to the first widget it sees.

For example, suppose we have a layout that looks like:
+-------------------+-----------------------+
|      CPU (1)      |        CPU (2)        |
+---------+---------+-------------+---------+
| Process | CPU (3) | Temperature | CPU (4) |
+---------+---------+-------------+---------+

Setting '--default_widget_type Temp' will make the Temperature
widget selected by default.

Supported widget names:
+--------------------------+
|            cpu           |
+--------------------------+
|        mem, memory       |
+--------------------------+
|       net, network       |
+--------------------------+
| proc, process, processes |
+--------------------------+
|     temp, temperature    |
+--------------------------+
|           disk           |
+--------------------------+
|       batt, battery      |
+--------------------------+
"
    }
    #[cfg(not(feature = "battery"))]
    {
        "\
Sets which widget type to use as the default widget.
For the default layout, this defaults to the 'process' widget.
For a custom layout, it defaults to the first widget it sees.

For example, suppose we have a layout that looks like:
+-------------------+-----------------------+
|      CPU (1)      |        CPU (2)        |
+---------+---------+-------------+---------+
| Process | CPU (3) | Temperature | CPU (4) |
+---------+---------+-------------+---------+

Setting '--default_widget_type Temp' will make the Temperature
widget selected by default.

Supported widget names:
+--------------------------+
|            cpu           |
+--------------------------+
|        mem, memory       |
+--------------------------+
|       net, network       |
+--------------------------+
| proc, process, processes |
+--------------------------+
|     temp, temperature    |
+--------------------------+
|           disk           |
+--------------------------+
"
    }
};

pub fn get_matches() -> ArgMatches {
    build_app().get_matches()
}

trait CommandBuilder {
    fn general_args(self) -> Self;

    fn style_args(self) -> Self;

    fn temperature_args(self) -> Self;

    fn process_args(self) -> Self;

    fn cpu_args(self) -> Self;

    fn mem_args(self) -> Self;

    fn network_args(self) -> Self;

    fn battery_args(self) -> Self;

    fn gpu_args(self) -> Self;

    fn other(self) -> Self;
}

impl CommandBuilder for Command {
    fn general_args(self) -> Command {
        const HEADING: &str = "General Options";

        let autohide_time = Arg::new("autohide_time")
            .long("autohide_time")
            .action(ArgAction::SetTrue)
            .help("Temporarily shows the time scale in graphs.")
            .long_help(
                "Automatically hides the time scale in graphs after being shown for \
            a brief moment when zoomed in/out. If time is disabled via --hide_time \
            then this will have no effect.",
            )
            .help_heading(HEADING);

        let basic = Arg::new("basic")
            .short('b')
            .long("basic")
            .action(ArgAction::SetTrue)
            .help("Hides graphs and uses a more basic look.")
            .long_help(
                "Hides graphs and uses a more basic look. Design is largely inspired by htop's.",
            )
            .help_heading(HEADING);

        let disable_click = Arg::new("disable_click")
            .long("disable_click")
            .action(ArgAction::SetTrue)
            .help("Disables mouse clicks.")
            .long_help("Disables mouse clicks from interacting with the program.")
            .help_heading(HEADING);

        let dot_marker = Arg::new("dot_marker")
            .short('m')
            .long("dot_marker")
            .action(ArgAction::SetTrue)
            .help("Uses a dot marker for graphs.")
            .long_help("Uses a dot marker for graphs as opposed to the default braille marker.")
            .help_heading(HEADING);

        let hide_table_gap = Arg::new("hide_table_gap")
            .long("hide_table_gap")
            .action(ArgAction::SetTrue)
            .help("Hides spacing between table headers and entries.")
            .long_help("Hides the spacing between table headers and entries.")
            .help_heading(HEADING);

        let hide_time = Arg::new("hide_time")
            .long("hide_time")
            .action(ArgAction::SetTrue)
            .help("Hides the time scale.")
            .long_help("Completely hides the time scale from being shown.")
            .help_heading(HEADING);

        let left_legend = Arg::new("left_legend")
            .short('l')
            .long("left_legend")
            .action(ArgAction::SetTrue)
            .help("Puts the CPU chart legend to the left side.")
            .long_help("Puts the CPU chart legend to the left side rather than the right side.")
            .help_heading(HEADING);

        let show_table_scroll_position = Arg::new("show_table_scroll_position")
            .long("show_table_scroll_position")
            .action(ArgAction::SetTrue)
            .help("Shows the scroll position tracker in table widgets.")
            .long_help(
                "Shows the list scroll position tracker in the widget title for table widgets.",
            )
            .help_heading(HEADING);

        let config_location = Arg::new("config_location")
            .short('C')
            .long("config")
            .action(ArgAction::Set)
            .value_name("CONFIG PATH")
            .help("Sets the location of the config file.")
            .long_help(
                "Sets the location of the config file. Expects a config file in the TOML format. \
                If it doesn't exist, one is created.",
            )
            .value_hint(ValueHint::AnyPath)
            .help_heading(HEADING);

        let default_time_value = Arg::new("default_time_value")
            .short('t')
            .long("default_time_value")
            .action(ArgAction::Set)
            .value_name("TIME")
            .help("Default time value for graphs.")
            .long_help(
                "Default time value for graphs. Takes a number in milliseconds or a human duration (e.g. 60s). The minimum time is 30s, and the default is 60s.",
            )
            .help_heading(HEADING);

        // TODO: Charts are broken in the manpage
        let default_widget_count = Arg::new("default_widget_count")
            .long("default_widget_count")
            .action(ArgAction::Set)
            .requires_all(["default_widget_type"])
            .value_name("INT")
            .help("Sets the n'th selected widget type as the default.")
            .long_help(
                "\
Sets the n'th selected widget type to use as the default widget.
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
use CPU (3) as the default instead.
    ",
            )
            .help_heading(HEADING);

        let default_widget_type = Arg::new("default_widget_type")
            .long("default_widget_type")
            .action(ArgAction::Set)
            .value_name("WIDGET TYPE")
            .help("Sets the default widget type, use --help for info.")
            .long_help(DEFAULT_WIDGET_TYPE_STR)
            .help_heading(HEADING);

        let expanded_on_startup = Arg::new("expanded_on_startup")
            .short('e')
            .long("expanded")
            .action(ArgAction::SetTrue)
            .help("Expand the default widget upon starting the app.")
            .long_help("Expand the default widget upon starting the app. Same as pressing \"e\" inside the app. Use with \"default_widget_type\" and \"default_widget_count\" to select desired expanded widget. This flag has no effect in basic mode (--basic)")
            .help_heading(HEADING);

        let rate = Arg::new("rate")
            .short('r')
            .long("rate")
            .action(ArgAction::Set)
            .value_name("TIME")
            .help("Sets the data refresh rate.")
            .long_help("Sets the data refresh rate. Takes a number in milliseconds or a human duration (e.g. 5s). The minimum is 250ms, and defaults to 1000ms. Smaller values may take more computer resources.")
            .help_heading(HEADING);

        let time_delta = Arg::new("time_delta")
            .short('d')
            .long("time_delta")
            .action(ArgAction::Set)
            .value_name("TIME")
            .help("The amount of time changed upon zooming.")
            .long_help("The amount of time changed when zooming in/out. Takes a number in milliseconds or a human duration (e.g. 30s). The minimum is 1s, and defaults to 15s.")
            .help_heading(HEADING);

        let retention = Arg::new("retention")
            .long("retention")
            .action(ArgAction::Set)
            .value_name("TIME")
            .help("The timespan of data stored.")
            .long_help("How much data is stored at once in terms of time. Takes a number in milliseconds or a human duration (e.g. 20m), with a minimum of 1 minute. Note higher values will take up more memory. Defaults to 10 minutes.")
            .help_heading(HEADING);

        let mut args = [
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
        ];
        args.sort_unstable();

        self.args(args)
    }

    fn style_args(self) -> Command {
        const HEADING: &str = "Style Options";

        // TODO: File an issue with manpage, it cannot render charts correctly.
        let color = Arg::new("color")
            .long("color")
            .action(ArgAction::Set)
            .value_name("COLOR SCHEME")
            .value_parser(PossibleValuesParser::new([
                "default",
                "default-light",
                "gruvbox",
                "gruvbox-light",
                "nord",
                "nord-light",
            ]))
            .hide_possible_values(true)
            .help("Use a color scheme, use --help for info.")
            .long_help(
                "\
Use a pre-defined color scheme. Currently supported values are:
+------------------------------------------------------------+
| default                                                    |
+------------------------------------------------------------+
| default-light (default but for use with light backgrounds) |
+------------------------------------------------------------+
| gruvbox (a bright theme with 'retro groove' colors)        |
+------------------------------------------------------------+
| gruvbox-light (gruvbox but for use with light backgrounds) |
+------------------------------------------------------------+
| nord (an arctic, north-bluish color palette)               |
+------------------------------------------------------------+
| nord-light (nord but for use with light backgrounds)       |
+------------------------------------------------------------+
Defaults to \"default\".
    ",
            )
            .help_heading(HEADING);

        self.arg(color)
    }

    fn temperature_args(self) -> Command {
        const HEADING: &str = "Temperature Options";

        let celsius = Arg::new("celsius")
            .short('c')
            .long("celsius")
            .action(ArgAction::SetTrue)
            .help("Use Celsius as the temperature unit.")
            .long_help("Use Celsius as the temperature unit. This is the default option.")
            .help_heading(HEADING);

        let fahrenheit = Arg::new("fahrenheit")
            .short('f')
            .long("fahrenheit")
            .action(ArgAction::SetTrue)
            .help("Use Fahrenheit as the temperature unit.")
            .help_heading(HEADING);

        let kelvin = Arg::new("kelvin")
            .short('k')
            .long("kelvin")
            .action(ArgAction::SetTrue)
            .help("Use Kelvin as the temperature unit.")
            .help_heading(HEADING);

        let temperature_group = ArgGroup::new("TEMPERATURE_TYPE").args([
            celsius.get_id(),
            fahrenheit.get_id(),
            kelvin.get_id(),
        ]);

        let args = [celsius, fahrenheit, kelvin];

        self.args(args).group(temperature_group)
    }

    fn process_args(self) -> Command {
        const HEADING: &str = "Process Options";

        let case_sensitive = Arg::new("case_sensitive")
            .short('S')
            .long("case_sensitive")
            .action(ArgAction::SetTrue)
            .help("Enables case sensitivity by default.")
            .long_help("When searching for a process, enables case sensitivity by default.")
            .help_heading(HEADING);

        let current_usage = Arg::new("current_usage")
            .short('u')
            .long("current_usage")
            .action(ArgAction::SetTrue)
            .help("Sets process CPU% to be based on current CPU%.")
            .long_help("Sets process CPU% usage to be based on the current system CPU% usage rather than total CPU usage.")
            .help_heading(HEADING);

        let unnormalized_cpu = Arg::new("unnormalized_cpu")
            .short('n')
            .long("unnormalized_cpu")
            .action(ArgAction::SetTrue)
            .help("Show process CPU% usage without normalizing over the number of cores.")
            .long_help(
                "Shows all process CPU% usage without averaging over the number of CPU cores in the system.",
            )
            .help_heading(HEADING);

        let group_processes = Arg::new("group_processes")
            .short('g')
            .long("group_processes")
            .action(ArgAction::SetTrue)
            .help("Groups processes with the same name by default.")
            .long_help("Groups processes with the same name by default.")
            .help_heading(HEADING);

        let process_command = Arg::new("process_command")
            .long("process_command")
            .action(ArgAction::SetTrue)
            .help("Show processes as their commands by default.")
            .long_help("Show processes as their commands by default in the process widget.")
            .help_heading(HEADING);

        let regex = Arg::new("regex")
            .short('R')
            .long("regex")
            .action(ArgAction::SetTrue)
            .help("Enables regex by default.")
            .long_help("When searching for a process, enables regex by default.")
            .help_heading(HEADING);

        let disable_advanced_kill = Arg::new("disable_advanced_kill")
            .long("disable_advanced_kill")
            .action(ArgAction::SetTrue)
            .help("Hides advanced process killing.")
            .long_help("Hides advanced options to stop a process on Unix-like systems. The only option shown is 15 (TERM).")
            .help_heading(HEADING);

        let whole_word = Arg::new("whole_word")
            .short('W')
            .long("whole_word")
            .action(ArgAction::SetTrue)
            .help("Enables whole-word matching by default.")
            .long_help(
                "When searching for a process, return results that match the entire query by default.",
            )
            .help_heading(HEADING);

        let tree = Arg::new("tree")
            .short('T')
            .long("tree")
            .action(ArgAction::SetTrue)
            .help("Defaults the process widget be in tree mode.")
            .long_help("Defaults to showing the process widget in tree mode.")
            .help_heading(HEADING);

        let mut args = [
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
        args.sort_unstable();

        self.args(args)
    }

    fn cpu_args(self) -> Command {
        const HEADING: &str = "CPU Options";

        let hide_avg_cpu = Arg::new("hide_avg_cpu")
            .short('a')
            .long("hide_avg_cpu")
            .action(ArgAction::SetTrue)
            .help("Hides the average CPU usage.")
            .long_help("Hides the average CPU usage from being shown.")
            .help_heading(HEADING);

        self.arg(hide_avg_cpu)
    }

    fn mem_args(self) -> Command {
        const HEADING: &str = "Memory Options";

        let mem_as_value = Arg::new("mem_as_value")
            .long("mem_as_value")
            .action(ArgAction::SetTrue)
            .help("Defaults to showing process memory usage by value.")
            .long_help("Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage.")
            .help_heading(HEADING);

        #[cfg(not(target_os = "windows"))]
        {
            let enable_cache_memory = Arg::new("enable_cache_memory")
                .long("enable_cache_memory")
                .action(ArgAction::SetTrue)
                .help("Enable collecting and displaying cache and buffer memory.")
                .help_heading(HEADING);

            self.args([mem_as_value, enable_cache_memory])
        }
        #[cfg(target_os = "windows")]
        {
            self.arg(mem_as_value)
        }
    }

    fn network_args(self) -> Command {
        const HEADING: &str = "Network Options";

        let use_old_network_legend = Arg::new("use_old_network_legend")
            .long("use_old_network_legend")
            .action(ArgAction::SetTrue)
            .help("DEPRECATED - uses a separate network legend.")
            .long_help(
                "DEPRECATED - uses an older (pre-0.4), separate network widget legend. This display is not \
                tested anymore and could be broken.",
            )
            .help_heading(HEADING);

        let network_use_bytes = Arg::new("network_use_bytes")
            .long("network_use_bytes")
            .action(ArgAction::SetTrue)
            .help("Displays the network widget using bytes.")
            .long_help("Displays the network widget using bytes. Defaults to bits.")
            .help_heading(HEADING);

        let network_use_log = Arg::new("network_use_log")
            .long("network_use_log")
            .action(ArgAction::SetTrue)
            .help("Displays the network widget with a log scale.")
            .long_help("Displays the network widget with a log scale. Defaults to a non-log scale.")
            .help_heading(HEADING);

        let network_use_binary_prefix = Arg::new("network_use_binary_prefix")
            .long("network_use_binary_prefix")
            .action(ArgAction::SetTrue)
            .help("Displays the network widget with binary prefixes.")
            .long_help(
                "Displays the network widget with binary prefixes (i.e. kibibits, mebibits) rather than a decimal prefix (i.e. kilobits, megabits). Defaults to decimal prefixes.",
            )
            .help_heading(HEADING);

        let mut args = [
            use_old_network_legend,
            network_use_bytes,
            network_use_log,
            network_use_binary_prefix,
        ];

        args.sort_unstable();

        self.args(args)
    }

    fn battery_args(self) -> Command {
        #[cfg(feature = "battery")]
        {
            let battery = Arg::new("battery")
            .long("battery")
            .action(ArgAction::SetTrue)
            .help("Shows the battery widget.")
            .long_help(
                "Shows the battery widget in default or basic mode. No effect on custom layouts.",
            )
            .help_heading("Battery Options");

            self.arg(battery)
        }
        #[cfg(not(feature = "battery"))]
        {
            self
        }
    }

    fn gpu_args(self) -> Command {
        #[cfg(feature = "gpu")]
        {
            let enable_gpu = Arg::new("enable_gpu")
                .long("enable_gpu")
                .action(ArgAction::SetTrue)
                .help("Enable collecting and displaying GPU usage.")
                .help_heading("GPU Options");

            self.arg(enable_gpu)
        }
        #[cfg(not(feature = "gpu"))]
        {
            self
        }
    }

    fn other(self) -> Command {
        const HEADING: &str = "Other Options";

        let version = Arg::new("version")
            .short('V')
            .long("version")
            .action(ArgAction::Version)
            .help("Prints version information.")
            .help_heading(HEADING);

        let help = Arg::new("help")
            .short('h')
            .long("help")
            .action(ArgAction::Help)
            .help("Prints help (see more with '--help').")
            .help_heading(HEADING);

        self.args([version, help])
    }
}

pub fn build_app() -> Command {
    const TEMPLATE: &str = include_str!("./args.template");
    const USAGE: &str = "btm [OPTIONS]";
    const VERSION: &str = match option_env!("NIGHTLY_VERSION") {
        Some(nightly_version) => nightly_version,
        None => crate_version!(),
    };

    Command::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .disable_help_flag(true)
        .disable_version_flag(true)
        .color(ColorChoice::Auto)
        .help_template(TEMPLATE)
        .override_usage(USAGE)
        .version(VERSION)
        .general_args()
        .style_args()
        .temperature_args()
        .process_args()
        .cpu_args()
        .mem_args()
        .network_args()
        .battery_args()
        .gpu_args()
        .other()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        build_app().debug_assert();
    }

    #[test]
    fn help_heading_no_default() {
        let mut app = build_app();
        let help_str = app.render_help();

        assert!(
            !help_str.to_string().contains("\nOptions:\n"),
            "the default 'Options' heading should not exist; if it does then an argument is missing a help heading."
        );
    }
}
