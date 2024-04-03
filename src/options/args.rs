//! Argument parsing via clap + config files.
//!
//! Note that you probably want to keep this as a single file so the build script doesn't
//! trip all over itself.

// TODO: New sections are misaligned! See if we can get that fixed.
// TODO: This might need some more work when we do config screens. For example, we can't just merge args + config,
// since we need to know the state of the config, the overwriting args, and adjust the calculated app settings as
// they change.

use clap::*;
use indoc::indoc;
use serde::Deserialize;

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

macro_rules! set_if_some {
    ($name:ident, $curr:expr, $new:expr) => {
        if $new.$name.is_some() {
            $curr.$name = $new.$name.clone();
        }
    };
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum StringOrNum {
    String(String),
    Num(u64),
}

impl From<&str> for StringOrNum {
    fn from(value: &str) -> Self {
        match value.parse::<u64>() {
            Ok(parsed) => StringOrNum::Num(parsed),
            Err(_) => StringOrNum::String(value.to_owned()),
        }
    }
}

impl From<u64> for StringOrNum {
    fn from(value: u64) -> Self {
        StringOrNum::Num(value)
    }
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

impl BottomArgs {
    /// Returns the config path if it is set.
    #[inline]
    pub fn config_path(&self) -> Option<&str> {
        self.general.config_location.as_deref()
    }
}

/// General arguments/config options.
#[derive(Args, Clone, Debug, Default, Deserialize)]
#[command(next_help_heading = "General Options", rename_all = "snake_case")]
pub(crate) struct GeneralArgs {
    #[arg(
        long,
        help = "Temporarily shows the time scale in graphs.",
        long_help = "Automatically hides the time scale in graphs after being shown for a brief moment when zoomed \
                    in/out. If time is disabled via --hide_time then this will have no effect."
    )]
    pub(crate) autohide_time: Option<bool>,

    #[arg(
        short = 'b',
        long,
        help = "Hides graphs and uses a more basic look.",
        long_help = "Hides graphs and uses a more basic look. Design is largely inspired by htop's."
    )]
    pub(crate) basic: Option<bool>,

    #[arg(
        short = 'C',
        long,
        value_name = "PATH",
        help = "Sets the location of the config file.",
        long_help = "Sets the location of the config file. Expects a config file in the TOML format. \
                    If it doesn't exist, a default config file is created at the path. If no path is provided,
                    the default config location will be used."
    )]
    #[serde(skip)]
    pub(crate) config_location: Option<String>,

    #[arg(
        short = 't',
        long,
        value_name = "TIME",
        help = "Default time value for graphs.",
        long_help = "Default time value for graphs. Either a number in milliseconds or a 'human duration' \
                    (e.g. 60s, 10m). Defaults to 60s, must be at least 30s."
    )]
    pub(crate) default_time_value: Option<StringOrNum>,

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
    pub(crate) rate: Option<StringOrNum>,

    #[arg(
        long,
        value_name = "TIME",
        help = "How far back data will be stored up to.",
        long_help = "How far back data will be stored up to. Either a number in milliseconds or a 'human duration' \
                    (e.g. 10m, 1h). Defaults to 10 minutes, and must be at least  1 minute. Larger values \
                    may result in higher memory usage."
    )]
    pub(crate) retention: Option<StringOrNum>,

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
    pub(crate) time_delta: Option<StringOrNum>,
}

impl GeneralArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        set_if_some!(autohide_time, self, other);
        set_if_some!(basic, self, other);
        set_if_some!(config_location, self, other);
        set_if_some!(default_time_value, self, other);
        set_if_some!(default_widget_count, self, other);
        set_if_some!(default_widget_type, self, other);
        set_if_some!(disable_click, self, other);
        set_if_some!(dot_marker, self, other);
        set_if_some!(expanded, self, other);
        set_if_some!(hide_time, self, other);
        set_if_some!(rate, self, other);
        set_if_some!(retention, self, other);
        set_if_some!(show_table_scroll_position, self, other);
        set_if_some!(time_delta, self, other);
    }
}

/// Process arguments/config options.
#[derive(Args, Clone, Debug, Default, Deserialize)]
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

impl ProcessArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        set_if_some!(case_sensitive, self, other);
        set_if_some!(current_usage, self, other);
        set_if_some!(disable_advanced_kill, self, other);
        set_if_some!(group_processes, self, other);
        set_if_some!(mem_as_value, self, other);
        set_if_some!(process_command, self, other);
        set_if_some!(regex, self, other);
        set_if_some!(tree, self, other);
        set_if_some!(unnormalized_cpu, self, other);
        set_if_some!(whole_word, self, other);
    }
}

/// Temperature arguments/config options.
#[derive(Args, Clone, Debug, Default, Deserialize)]
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
    #[serde(skip)]
    pub(crate) celsius: bool,

    #[arg(
        short = 'f',
        long,
        group = "temperature_unit",
        help = "Use Fahrenheit as the temperature unit."
    )]
    #[serde(skip)]
    pub(crate) fahrenheit: bool,

    #[arg(
        short = 'k',
        long,
        group = "temperature_unit",
        help = "Use Kelvin as the temperature unit."
    )]
    #[serde(skip)]
    pub(crate) kelvin: bool,
}

impl TemperatureArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        self.celsius |= other.celsius;
        self.fahrenheit |= other.fahrenheit;
        self.kelvin |= other.kelvin;
    }
}

/// The default selection of the CPU widget. If the given selection is invalid,
/// we will fall back to all.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CpuDefault {
    #[default]
    All,
    #[serde(alias = "avg")]
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
#[derive(Args, Clone, Debug, Default, Deserialize)]
#[command(next_help_heading = "CPU Options", rename_all = "snake_case")]
pub(crate) struct CpuArgs {
    #[arg(
        long,
        help = "Sets which CPU entry is selected by default.",
        value_name = "ENTRY",
        value_parser = ["all", "avg"],
        default_value = "all"
    )]
    #[serde(default)]
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

impl CpuArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        // set_if_some!(default_cpu_entry, self, other);
        set_if_some!(hide_avg_cpu, self, other);
        set_if_some!(left_legend, self, other);
    }
}

/// Memory argument/config options.
#[derive(Args, Clone, Debug, Default, Deserialize)]
#[command(next_help_heading = "Memory Options", rename_all = "snake_case")]
pub(crate) struct MemoryArgs {
    #[cfg(not(target_os = "windows"))]
    #[arg(
        long,
        help = "Enables collecting and displaying cache and buffer memory."
    )]
    pub(crate) enable_cache_memory: Option<bool>,
}

impl MemoryArgs {
    // Lint needed because of target_os.
    #[allow(unused_variables)]
    pub(crate) fn merge(&mut self, other: &Self) {
        #[cfg(not(target_os = "windows"))]
        set_if_some!(enable_cache_memory, self, other);
    }
}

/// Network arguments/config options.
#[derive(Args, Clone, Debug, Default, Deserialize)]
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

impl NetworkArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        set_if_some!(network_use_bytes, self, other);
        set_if_some!(network_use_binary_prefix, self, other);
        set_if_some!(network_use_log, self, other);
        set_if_some!(use_old_network_legend, self, other);
    }
}

/// Battery arguments/config options.
#[cfg(feature = "battery")]
#[derive(Args, Clone, Debug, Default, Deserialize)]
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

impl BatteryArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        set_if_some!(battery, self, other);
    }
}

/// GPU arguments/config options.
#[cfg(feature = "gpu")]
#[derive(Args, Clone, Debug, Default, Deserialize)]
#[command(next_help_heading = "GPU Options", rename_all = "snake_case")]
pub(crate) struct GpuArgs {
    #[arg(long, help = "Enables collecting and displaying GPU usage.")]
    pub(crate) enable_gpu: Option<bool>,
}

#[cfg(feature = "gpu")]
impl GpuArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        set_if_some!(enable_gpu, self, other);
    }
}

/// Style arguments/config options.
#[derive(Args, Clone, Debug, Default, Deserialize)]
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

impl StyleArgs {
    pub(crate) fn merge(&mut self, other: &Self) {
        set_if_some!(color, self, other);
    }
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

/// Returns a [`BottomArgs`].
pub fn get_args() -> BottomArgs {
    BottomArgs::parse()
}

/// Returns an [`Command`] based off of [`BottomArgs`].
#[allow(dead_code)]
pub(crate) fn build_cmd() -> Command {
    BottomArgs::command()
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
