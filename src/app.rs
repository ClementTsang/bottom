pub mod data_farmer;
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};

pub use data_farmer::*;

pub mod data_harvester;
use data_harvester::temperature;

pub mod event;

pub mod filter;
pub use filter::*;

pub mod layout_manager;
use layout_manager::*;

pub mod widgets;
pub use widgets::*;

mod process_killer;
pub mod query;

mod frozen_state;
use frozen_state::FrozenState;

use crate::{
    canvas::Painter,
    constants,
    tuice::{Application, Row},
    units::data_units::DataUnit,
    Pid,
};

use anyhow::Result;
use indextree::{Arena, NodeId};
use rustc_hash::FxHashMap;

// FIXME: Move this!
#[derive(Debug, Clone)]
pub enum AxisScaling {
    Log,
    Linear,
}

#[derive(Clone, Default, Debug)]
pub struct UsedWidgets {
    pub use_cpu: bool,
    pub use_mem: bool,
    pub use_net: bool,
    pub use_proc: bool,
    pub use_disk: bool,
    pub use_temp: bool,
    pub use_battery: bool,
}

impl UsedWidgets {
    pub fn add(&mut self, widget_type: &BottomWidgetType) {
        match widget_type {
            BottomWidgetType::Cpu | BottomWidgetType::BasicCpu => {
                self.use_cpu = true;
            }
            BottomWidgetType::Mem | BottomWidgetType::BasicMem => {
                self.use_mem = true;
            }
            BottomWidgetType::Net | BottomWidgetType::BasicNet => {
                self.use_net = true;
            }
            BottomWidgetType::Proc => {
                self.use_proc = true;
            }
            BottomWidgetType::Temp => {
                self.use_temp = true;
            }
            BottomWidgetType::Disk => {
                self.use_disk = true;
            }
            BottomWidgetType::Battery => {
                self.use_battery = true;
            }
            _ => {}
        }
    }
}

/// AppConfigFields is meant to cover basic fields that would normally be set
/// by config files or launch options.
#[derive(Debug)]
pub struct AppConfigFields {
    pub update_rate_in_milliseconds: u64,
    pub temperature_type: temperature::TemperatureType,
    pub use_dot: bool,
    pub left_legend: bool,
    pub show_average_cpu: bool,
    pub use_current_cpu_total: bool,
    pub use_basic_mode: bool,
    pub default_time_value: u64,
    pub time_interval: u64,
    pub hide_time: bool,
    pub autohide_time: bool,
    pub use_old_network_legend: bool,
    pub table_gap: bool,
    pub disable_click: bool,
    pub no_write: bool,
    pub show_table_scroll_position: bool,
    pub is_advanced_kill: bool,
    pub network_unit_type: DataUnit,
    pub network_scale_type: AxisScaling,
    pub network_use_binary_prefix: bool,
}

#[derive(PartialEq, Eq)]
enum CurrentScreen {
    Main,
    Expanded,
    Help,
    Delete,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Debug)]
pub enum AppMessages {
    Update(Box<data_harvester::Data>),
    OpenHelp,
    KillProcess { to_kill: Vec<Pid> },
    ToggleFreeze,
    Clean,
    Stop,
}

pub struct AppState {
    pub data_collection: DataCollection,

    pub used_widgets: UsedWidgets,
    pub filters: DataFilters,
    pub app_config_fields: AppConfigFields,

    // --- NEW STUFF ---
    pub selected_widget: NodeId,
    pub widget_lookup_map: FxHashMap<NodeId, BottomWidget>,
    pub layout_tree: Arena<LayoutNode>,
    pub layout_tree_root: NodeId,

    frozen_state: FrozenState,
    current_screen: CurrentScreen,
    painter: Painter,
    terminator: Arc<AtomicBool>,
}

impl AppState {
    /// Creates a new [`AppState`].
    pub fn new(
        app_config_fields: AppConfigFields, filters: DataFilters,
        layout_tree_output: LayoutCreationOutput, painter: Painter,
    ) -> Result<Self> {
        let LayoutCreationOutput {
            layout_tree,
            root: layout_tree_root,
            widget_lookup_map,
            selected: selected_widget,
            used_widgets,
        } = layout_tree_output;

        Ok(Self {
            app_config_fields,
            filters,
            used_widgets,
            selected_widget,
            widget_lookup_map,
            layout_tree,
            layout_tree_root,
            painter,

            // Use defaults.
            data_collection: Default::default(),
            frozen_state: Default::default(),
            current_screen: Default::default(),

            terminator: Self::register_terminator()?,
        })
    }

    fn register_terminator() -> Result<Arc<AtomicBool>> {
        let it = Arc::new(AtomicBool::new(false));
        let it_clone = it.clone();
        ctrlc::set_handler(move || {
            it_clone.store(true, SeqCst);
        })?;

        Ok(it)
    }

    fn set_current_screen(&mut self, screen_type: CurrentScreen) {
        if self.current_screen == screen_type {
            self.current_screen = screen_type;
            // TODO: Redraw
        }
    }
}

impl Application for AppState {
    type Message = AppMessages;

    fn update(&mut self, message: Self::Message) {
        match message {
            AppMessages::Update(new_data) => {
                self.data_collection.eat_data(new_data);
            }
            AppMessages::OpenHelp => {
                self.set_current_screen(CurrentScreen::Help);
            }
            AppMessages::KillProcess { to_kill } => {}
            AppMessages::ToggleFreeze => {
                self.frozen_state.toggle(&self.data_collection);
            }
            AppMessages::Clean => {
                self.data_collection
                    .clean_data(constants::STALE_MAX_MILLISECONDS);
            }
            AppMessages::Stop => {
                self.terminator.store(true, SeqCst);
            }
        }
    }

    fn is_terminated(&self) -> bool {
        self.terminator.load(SeqCst)
    }

    fn view(
        &mut self,
    ) -> Box<dyn crate::tuice::Component<Self::Message, crate::tuice::CrosstermBackend>> {
        Box::new(Row::with_children(vec![crate::tuice::TextTable::new(
            vec!["A", "B", "C"],
        )]))
    }

    fn destroy(&mut self) {
        // TODO: Eventually move some thread logic into the app creation, and destroy here?
    }

    fn global_event_handler(
        &mut self, event: crate::tuice::Event, _messages: &mut Vec<Self::Message>,
    ) {
        use crate::tuice::Event;
        match event {
            Event::Keyboard(_) => {}
            Event::Mouse(_) => {}
        }
    }
}
