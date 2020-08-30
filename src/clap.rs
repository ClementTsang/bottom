use clap::*;

pub fn get_matches() -> clap::ArgMatches<'static> {
    build_matches().get_matches()
}

pub fn build_matches() -> App<'static, 'static> {
    clap_app!(app =>
        (name: crate_name!())
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg HIDE_AVG_CPU: -a --hide_avg_cpu "Hides the average CPU usage.")
        (@arg DOT_MARKER: -m --dot_marker "Use a dot marker instead of the default braille marker.")
        (@group TEMPERATURE_TYPE =>
            (@arg KELVIN : -k --kelvin "Sets the temperature type to Kelvin.")
            (@arg FAHRENHEIT : -f --fahrenheit "Sets the temperature type to Fahrenheit.")
            (@arg CELSIUS : -c --celsius "Sets the temperature type to Celsius.  This is the default option.")
        )
        (@arg RATE_MILLIS: -r --rate +takes_value "Sets a refresh rate in milliseconds; the minimum is 250ms, defaults to 1000ms.  Smaller values may take more resources.")
        (@arg LEFT_LEGEND: -l --left_legend "Puts external chart legends on the left side rather than the default right side.")
        (@arg USE_CURR_USAGE: -u --current_usage "Within Linux, sets a process' CPU usage to be based on the total current CPU usage, rather than assuming 100% usage.")
        (@arg CONFIG_LOCATION: -C --config +takes_value "Sets the location of the config file.  Expects a config file in the TOML format. If it doesn't exist, one is created.")
        (@arg BASIC_MODE: -b --basic "Hides graphs and uses a more basic look")
        (@arg GROUP_PROCESSES: -g --group "Groups processes with the same name together on launch.")
        (@arg CASE_SENSITIVE: -S --case_sensitive "Match case when searching by default.")
        (@arg WHOLE_WORD: -W --whole_word "Match whole word when searching by default.")
        (@arg REGEX_DEFAULT: -R --regex "Use regex in searching by default.")
        (@arg DEFAULT_TIME_VALUE: -t --default_time_value +takes_value "Default time value for graphs in milliseconds; minimum is 30s, defaults to 60s.")
        (@arg TIME_DELTA: -d --time_delta +takes_value "The amount changed upon zooming in/out in milliseconds; minimum is 1s, defaults to 15s.")
        (@arg HIDE_TIME: --hide_time "Completely hide the time scaling")
        (@arg AUTOHIDE_TIME: --autohide_time "Automatically hide the time scaling in graphs after being shown for a brief moment when zoomed in/out.  If time is disabled via --hide_time then this will have no effect.")
        (@arg DEFAULT_WIDGET_TYPE: --default_widget_type +takes_value "The default widget type to select by default.")
        (@arg DEFAULT_WIDGET_COUNT: --default_widget_count +takes_value "Which number of the selected widget type to select, from left to right, top to bottom.  Defaults to 1.")
        (@arg USE_OLD_NETWORK_LEGEND: --use_old_network_legend "Use the older (pre-0.4) network widget legend.")
        (@arg HIDE_TABLE_GAP: --hide_table_gap "Hides the spacing between the table headers and entries.")
        (@arg BATTERY: --battery "Shows the battery widget in default or basic mode.  No effect on custom layouts.")
        (@arg DISABLE_CLICK: --disable_click "Disables mouse clicks from interacting with the program.")
    )
}
