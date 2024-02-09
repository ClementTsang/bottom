use indoc::indoc;
use serde::Deserialize;

use crate::args::GeneralArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GeneralConfig {
    #[serde(flatten)]
    pub(crate) args: GeneralArgs,
}

impl DefaultConfig for GeneralConfig {
    fn default_config() -> String {
        let s = indoc! {r##"
            # Temporarily shows the time scale in graphs. If time is disabled via --hide_time then this will
            # have no effect.
            # autohide_time = false
            
            # Hides graphs and uses a more basic look.
            # basic = false

            # Default time value for graphs. Either a number in milliseconds or a 'human duration'
            # (e.g. "60s", "10m"). Defaults to 60s, and must be at least 30s.
            # default_time_value = "60s"

            # Sets the n'th selected default widget type as the default. Requires `default_widget_type`
            # to be set to have any effect.
            # default_widget_count = 1

            # Sets which widget type to use as the default widget.
            # default_widget_type = "process"

            # Disables mouse clicks.
            # disable_click = false

            # Use a dot marker for graphs.
            # dot_marker = false

            # Expand the default widget upon starting the app. No effect on basic mode.
            # expanded = false

            # Hides spacing between table headers and entries.
            # hide_table_gap = false

            # Hides the time scale from being shown.
            # hide_time = false

            # Sets how often data is refreshed. Either a number in milliseconds or a 'human duration'
            # (e.g. "1s", "1m"). Defaults to 1s, and must be at least 250ms. Smaller values may result in
            # higher system resource usage.
            # rate = "1s"

            # How far back data will be stored up to. Either a number in milliseconds or a 'human duration'
            # (e.g. "10m", "1h"). Defaults to 10 minutes, and must be at least  1 minute. Larger values
            # may result in higher memory usage.
            # retention = "10m"

            # Show the current item entry position for table widgets.
            # show_table_scroll_position = false

            # How much time the x-axis shifts by each time you zoom in or out. Either a number in milliseconds or
            # a 'human duration' (e.g. "15s", "1m"). Defaults to 15 seconds.
            # time_delta = "15s"
        "##};

        s.to_string()
    }
}
