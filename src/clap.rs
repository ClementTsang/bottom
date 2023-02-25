use clap::builder::PossibleValuesParser;
use clap::*;

const TEMPLATE: &str = "\
{bin} {version}
{author}

{about}

USAGE:{usage}

FLAGS:
{options}";

const USAGE: &str = "
    btm [FLAG]";

const DEFAULT_WIDGET_TYPE_STR: &str = if cfg!(feature = "battery") {
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
} else {
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
};

pub fn get_matches() -> clap::ArgMatches {
    build_app().get_matches()
}

// TODO: Refactor this a bit, it's quite messy atm
// TODO: [DEBUG] Add a proper debugging solution.
pub fn build_app() -> Command<'static> {
    // Temps
    let kelvin = Arg::new("kelvin")
        .short('k')
        .long("kelvin")
        .help("Sets the temperature type to Kelvin.")
        .long_help("Sets the temperature type to Kelvin.");

    let fahrenheit = Arg::new("fahrenheit")
        .short('f')
        .long("fahrenheit")
        .help("Sets the temperature type to Fahrenheit.")
        .long_help("Sets the temperature type to Fahrenheit.");

    let celsius = Arg::new("celsius")
        .short('c')
        .long("celsius")
        .help("Sets the temperature type to Celsius.")
        .long_help("Sets the temperature type to Celsius. This is the default option.");

    // All flags. These are in alphabetical order
    let autohide_time = Arg::new("autohide_time")
        .long("autohide_time")
        .help("Temporarily shows the time scale in graphs.")
        .long_help(
            "Automatically hides the time scale in graphs after being shown for \
            a brief moment when zoomed in/out. If time is disabled via --hide_time \
            then this will have no effect.",
        );

    let basic = Arg::new("basic")
        .short('b')
        .long("basic")
        .help("Hides graphs and uses a more basic look.")
        .long_help(
            "Hides graphs and uses a more basic look. Design is largely inspired by htop's.",
        );

    let case_sensitive = Arg::new("case_sensitive")
        .short('S')
        .long("case_sensitive")
        .help("Enables case sensitivity by default.")
        .long_help("When searching for a process, enables case sensitivity by default.");

    let current_usage = Arg::new("current_usage")
        .short('u')
        .long("current_usage")
        .help("Sets process CPU% to be based on current CPU%.")
        .long_help("Sets process CPU% usage to be based on the current system CPU% usage rather than total CPU usage.");

    let unnormalized_cpu = Arg::new("unnormalized_cpu")
        .short('n')
        .long("unnormalized_cpu")
        .help("Show process CPU% without normalizing over the number of cores.")
        .long_help(
            "Shows process CPU usage without averaging over the number of CPU cores in the system.",
        );

    let disable_click = Arg::new("disable_click")
        .long("disable_click")
        .help("Disables mouse clicks.")
        .long_help("Disables mouse clicks from interacting with the program.");

    let dot_marker = Arg::new("dot_marker")
        .short('m')
        .long("dot_marker")
        .help("Uses a dot marker for graphs.")
        .long_help("Uses a dot marker for graphs as opposed to the default braille marker.");

    let group = Arg::new("group") // TODO: Rename this to something like "group_process", would be "breaking" though.
        .short('g')
        .long("group")
        .help("Groups processes with the same name by default.")
        .long_help("Groups processes with the same name by default.");

    let hide_avg_cpu = Arg::new("hide_avg_cpu")
        .short('a')
        .long("hide_avg_cpu")
        .help("Hides the average CPU usage.")
        .long_help("Hides the average CPU usage from being shown.");

    let hide_table_gap = Arg::new("hide_table_gap")
        .long("hide_table_gap")
        .help("Hides spacing between table headers and entries.")
        .long_help("Hides the spacing between table headers and entries.");

    let hide_time = Arg::new("hide_time")
        .long("hide_time")
        .help("Hides the time scale.")
        .long_help("Completely hides the time scale from being shown.");

    let process_command = Arg::new("process_command")
        .long("process_command")
        .help("Show processes as their commands by default.")
        .long_help("Show processes as their commands by default in the process widget.");

    let left_legend = Arg::new("left_legend")
        .short('l')
        .long("left_legend")
        .help("Puts the CPU chart legend to the left side.")
        .long_help("Puts the CPU chart legend to the left side rather than the right side.");

    let regex = Arg::new("regex")
        .short('R')
        .long("regex")
        .help("Enables regex by default.")
        .long_help("When searching for a process, enables regex by default.");

    let disable_advanced_kill = Arg::new("disable_advanced_kill")
        .long("disable_advanced_kill")
        .help("Hides advanced process killing.")
        .long_help("Hides advanced options to stop a process on Unix-like systems. The only option shown is 15 (TERM).");

    let show_table_scroll_position = Arg::new("show_table_scroll_position")
        .long("show_table_scroll_position")
        .help("Shows the scroll position tracker in table widgets.")
        .long_help("Shows the list scroll position tracker in the widget title for table widgets.");

    let use_old_network_legend = Arg::new("use_old_network_legend")
        .long("use_old_network_legend")
        .help("DEPRECATED - uses a separate network legend.")
        .long_help(
            "DEPRECATED - uses an older (pre-0.4), separate network widget legend. This display is not \
            tested anymore and could be broken.",
        );

    let whole_word = Arg::new("whole_word")
        .short('W')
        .long("whole_word")
        .help("Enables whole-word matching by default.")
        .long_help(
            "When searching for a process, return results that match the entire query by default.",
        );

    // All options. Again, alphabetical order.
    let config_location = Arg::new("config_location")
        .short('C')
        .long("config")
        .takes_value(true)
        .value_name("CONFIG PATH")
        .help("Sets the location of the config file.")
        .long_help(
            "Sets the location of the config file. Expects a config file in the TOML format. \
            If it doesn't exist, one is created.",
        );

    // TODO: Fix this, its broken in the manpage
    let color = Arg::new("color")
        .long("color")
        .takes_value(true)
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
        );

    let mem_as_value = Arg::new("mem_as_value")
        .long("mem_as_value")
        .help("Defaults to showing process memory usage by value.")
        .long_help("Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage.");

    let default_time_value = Arg::new("default_time_value")
        .short('t')
        .long("default_time_value")
        .takes_value(true)
        .value_name("MS")
        .help("Default time value for graphs in ms.")
        .long_help("Default time value for graphs in milliseconds. The minimum time is 30s (30000), and the default is 60s (60000).");

    // TODO: Fix this, its broken in the manpage
    let default_widget_count = Arg::new("default_widget_count")
        .long("default_widget_count")
        .takes_value(true)
        .requires_all(&["default_widget_type"])
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
        );

    let default_widget_type = Arg::new("default_widget_type")
        .long("default_widget_type")
        .takes_value(true)
        .value_name("WIDGET TYPE")
        .help("Sets the default widget type, use --help for info.")
        .long_help(DEFAULT_WIDGET_TYPE_STR);

    let expanded_on_startup = Arg::new("expanded_on_startup")
        .long("expanded")
        .short('e')
        .help("Expand the default widget upon starting the app.")
        .long_help("Expand the default widget upon starting the app. Same as pressing \"e\" inside the app. Use with \"default_widget_type\" and \"default_widget_count\" to select desired expanded widget. This flag has no effect in basic mode (--basic)");

    let rate = Arg::new("rate")
        .short('r')
        .long("rate")
        .takes_value(true)
        .value_name("MS")
        .help("Sets a refresh rate in ms.")
        .long_help("Sets a refresh rate in milliseconds. The minimum is 250ms, and defaults to 1000ms. Smaller values may take more computer resources.");

    let time_delta = Arg::new("time_delta")
        .short('d')
        .long("time_delta")
        .takes_value(true)
        .value_name("MS")
        .help("The amount in ms changed upon zooming.")
        .long_help("The amount of time in milliseconds changed when zooming in/out. The minimum is 1s (1000), and defaults to 15s (15000).");

    let tree = Arg::new("tree")
        .short('T')
        .long("tree")
        .help("Defaults the process widget be in tree mode.")
        .long_help("Defaults to showing the process widget in tree mode.");

    let network_use_bytes = Arg::new("network_use_bytes")
        .long("network_use_bytes")
        .help("Displays the network widget using bytes.")
        .long_help("Displays the network widget using bytes. Defaults to bits.");

    let network_use_log = Arg::new("network_use_log")
        .long("network_use_log")
        .help("Displays the network widget with a log scale.")
        .long_help("Displays the network widget with a log scale. Defaults to a non-log scale.");

    let network_use_binary_prefix = Arg::new("network_use_binary_prefix")
        .long("network_use_binary_prefix")
        .help("Displays the network widget with binary prefixes.")
        .long_help(
            "Displays the network widget with binary prefixes (i.e. kibibits, mebibits) rather than a decimal prefix (i.e. kilobits, megabits). Defaults to decimal prefixes.",
        );

    let retention = Arg::new("retention")
        .long("retention")
        .takes_value(true)
        .value_name("time")
        .help("The timespan of data kept.")
        .long_help("How much data is stored at once in terms of time. Takes in human-readable time spans (e.g. 10m, 1h), with a minimum of 1 minute. Note higher values will take up more memory. Defaults to 10 minutes.");

    const VERSION: &str = match option_env!("NIGHTLY_VERSION") {
        Some(nightly_version) => nightly_version,
        None => crate_version!(),
    };

    #[allow(unused_mut)]
    let mut app = Command::new(crate_name!())
        .version(VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .override_usage(USAGE)
        .help_template(TEMPLATE)
        .mut_arg("help", |a| {
            a.help("Prints help information. Use --help for info.")
        })
        .mut_arg("version", |a| a.help("Prints version information."))
        .arg(kelvin)
        .arg(fahrenheit)
        .arg(celsius)
        .group(ArgGroup::new("TEMPERATURE_TYPE").args(&["kelvin", "fahrenheit", "celsius"]))
        .arg(autohide_time)
        .arg(basic)
        .arg(case_sensitive)
        .arg(process_command)
        .arg(config_location)
        .arg(color)
        .arg(mem_as_value)
        .arg(default_time_value)
        .arg(default_widget_count)
        .arg(default_widget_type)
        .arg(disable_click)
        .arg(dot_marker)
        .arg(group)
        .arg(hide_avg_cpu)
        .arg(hide_table_gap)
        .arg(hide_time)
        .arg(show_table_scroll_position)
        .arg(left_legend)
        .arg(disable_advanced_kill)
        .arg(rate)
        .arg(regex)
        .arg(time_delta)
        .arg(tree)
        .arg(network_use_bytes)
        .arg(network_use_log)
        .arg(network_use_binary_prefix)
        .arg(current_usage)
        .arg(unnormalized_cpu)
        .arg(use_old_network_legend)
        .arg(whole_word)
        .arg(retention)
        .arg(expanded_on_startup);

    #[cfg(feature = "battery")]
    {
        let battery = Arg::new("battery")
            .long("battery")
            .help("Shows the battery widget.")
            .long_help(
                "Shows the battery widget in default or basic mode. No effect on custom layouts.",
            );
        app = app.arg(battery);
    }

    #[cfg(feature = "gpu")]
    {
        let enable_gpu_memory = Arg::new("enable_gpu_memory")
            .long("enable_gpu_memory")
            .help("Enable collecting and displaying GPU memory usage.");
        app = app.arg(enable_gpu_memory);
    }

    app
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        build_app().debug_assert();
    }
}
