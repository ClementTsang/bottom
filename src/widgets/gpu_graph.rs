use std::{borrow::Cow, num::NonZeroU16, time::Instant};

use crate::{
    app::AppConfigFields,
    canvas::{
        Painter,
        components::data_table::{
            Column, ColumnHeader, DataTable, DataTableColumn, DataTableProps, DataTableStyling,
            DataToCell,
        },
    },
    collection::agnostic_gpu::AgnosticGpuData,
    options::config::style::Styles,
};
use tui::widgets::Row;

pub enum GpuWidgetColumn {
    Gpu,
    Use,
    Vram,
}

impl ColumnHeader for GpuWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            GpuWidgetColumn::Gpu => "GPU".into(),
            GpuWidgetColumn::Use => "Use".into(),
            GpuWidgetColumn::Vram => "VRAM".into(),
        }
    }
}

pub enum GpuWidgetTableData {
    Entry {
        name: String,
        usage: f64,
        vram_used: u64,
        vram_total: u64,
    },
}

impl GpuWidgetTableData {
    pub fn from_gpu_data(data: &AgnosticGpuData) -> GpuWidgetTableData {
        GpuWidgetTableData::Entry {
            name: data.name.clone(),
            usage: data.load_percent,
            vram_used: data.memory_used,
            vram_total: data.memory_total,
        }
    }
}

impl DataToCell<GpuWidgetColumn> for GpuWidgetTableData {
    fn to_cell_text(
        &self, column: &GpuWidgetColumn, _calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        match self {
            GpuWidgetTableData::Entry {
                name,
                usage,
                vram_used,
                vram_total,
            } => match column {
                GpuWidgetColumn::Gpu => Some(name.clone().into()),
                GpuWidgetColumn::Use => Some(format!("{:.0}%", usage).into()),
                GpuWidgetColumn::Vram => Some(
                    format!(
                        "{:.1}/{:.1} GB",
                        *vram_used as f64 / 1024.0 / 1024.0 / 1024.0,
                        *vram_total as f64 / 1024.0 / 1024.0 / 1024.0
                    )
                    .into(),
                ),
            },
        }
    }

    #[inline(always)]
    fn style_row<'a>(&self, row: Row<'a>, _painter: &Painter) -> Row<'a> {
        row
    }

    fn column_widths<C: DataTableColumn<GpuWidgetColumn>>(
        _data: &[Self], _columns: &[C],
    ) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![1, 1, 1]
    }
}

/// The state of a GPU widget.
pub struct GpuWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    pub table: DataTable<GpuWidgetTableData, GpuWidgetColumn>,
}

impl GpuWidgetState {
    /// Create a new [`GpuWidgetState`].
    pub fn new(config: &AppConfigFields, current_display_time: u64, colours: &Styles) -> Self {
        const COLUMNS: [Column<GpuWidgetColumn>; 3] = [
            Column::soft(GpuWidgetColumn::Gpu, Some(0.4)),
            Column::soft(GpuWidgetColumn::Use, Some(0.2)),
            Column::soft(GpuWidgetColumn::Vram, Some(0.2)),
        ];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: false,
            show_current_entry_when_unfocused: true,
        };

        let styling = DataTableStyling::from_palette(colours);
        let table = DataTable::new(COLUMNS, props, styling);

        GpuWidgetState {
            current_display_time,
            autohide_timer: if config.autohide_time {
                Some(Instant::now())
            } else {
                None
            },
            table,
        }
    }

    /// Set the data for the widget's legend.
    pub fn set_legend_data(&mut self, data: &[AgnosticGpuData]) {
        self.table
            .set_data(data.iter().map(GpuWidgetTableData::from_gpu_data).collect());
    }
}
