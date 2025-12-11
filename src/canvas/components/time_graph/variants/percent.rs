//! A variant of a [`TimeGraph`] that expects data to be in a percentage format, from 0.0 to 100.0.

use std::borrow::Cow;

use tui::{layout::Constraint, symbols::Marker};

use crate::{
    app::{AppConfigFields, GraphStyle},
    canvas::components::time_graph::{
        AxisBound, ChartScaling, LegendPosition, TimeGraph, variants::get_border_style,
    },
    options::config::style::Styles,
};

/// Acts as a wrapper for a [`TimeGraph`] that expects data to be in a percentage format,
pub(crate) struct PercentTimeGraph<'a> {
    /// The total display range of the graph in milliseconds.
    ///
    /// TODO: Make this a [`std::time::Duration`].
    pub(crate) display_range: u64,

    /// Whether to hide the x-axis labels.
    pub(crate) hide_x_labels: bool,

    /// The app config fields.
    ///
    /// This is mostly used as a shared mutability workaround due to [`App`]
    /// being a giant state struct.
    pub(crate) app_config_fields: &'a AppConfigFields,

    /// The current widget selected by the app.
    ///
    /// This is mostly used as a shared mutability workaround due to [`App`]
    /// being a giant state struct.
    pub(crate) current_widget: u64,

    /// Whether the current widget is expanded.
    ///  
    /// This is mostly used as a shared mutability workaround due to [`App`]
    /// being a giant state struct.
    pub(crate) is_expanded: bool,

    /// The title of the graph.
    pub(crate) title: Cow<'a, str>,

    /// A reference to the styles.
    pub(crate) styles: &'a Styles,

    /// The widget ID corresponding to this graph.
    pub(crate) widget_id: u64,

    /// The position of the legend.
    pub(crate) legend_position: Option<LegendPosition>,

    /// The constraints for the legend.
    pub(crate) legend_constraints: Option<(Constraint, Constraint)>,

    /// The borders to draw.
    pub(crate) borders: tui::widgets::Borders,
}

impl<'a> PercentTimeGraph<'a> {
    /// Return the final [`TimeGraph`].
    pub fn build(self) -> TimeGraph<'a> {
        const Y_BOUNDS: AxisBound = AxisBound::Max(100.5);
        const Y_LABELS: [Cow<'static, str>; 2] = [Cow::Borrowed("  0%"), Cow::Borrowed("100%")];

        let x_min = -(self.display_range as f64);

        let marker = match self.app_config_fields.graph_style {
            GraphStyle::Dot => Marker::Dot,
            GraphStyle::Block => Marker::Block,
            GraphStyle::Filled => Marker::Braille,
            GraphStyle::Braille => Marker::Braille,
        };

        let graph_style = self.styles.graph_style;
        let border_style = get_border_style(self.styles, self.widget_id, self.current_widget);
        let title_style = self.styles.widget_title_style;
        let border_type = self.styles.border_type;

        TimeGraph {
            x_min,
            hide_x_labels: self.hide_x_labels,
            y_bounds: Y_BOUNDS,
            y_labels: &Y_LABELS,
            graph_style,
            border_style,
            border_type,
            title: self.title,
            is_selected: self.current_widget == self.widget_id,
            is_expanded: self.is_expanded,
            title_style,
            legend_position: self.legend_position,
            legend_constraints: self.legend_constraints,
            marker,
            scaling: ChartScaling::Linear,
            borders: self.borders,
        }
    }
}
