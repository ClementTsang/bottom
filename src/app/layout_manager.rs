use crate::error::{BottomError, Result};

/// Represents a more usable representation of the layout, derived from the
/// config.
#[derive(Debug)]
pub struct BottomLayout {
    pub rows: Vec<BottomRow>,
}

impl Default for BottomLayout {
    fn default() -> Self {
        BottomLayout {
            rows: vec![
                BottomRow {
                    ratio: 1,
                    children: vec![BottomCol {
                        ratio: 1,
                        children: vec![BottomWidget {
                            ratio: 1,
                            widget_type: BottomWidgetType::Cpu,
                        }],
                    }],
                },
                BottomRow {
                    ratio: 1,
                    children: vec![
                        BottomCol {
                            ratio: 4,
                            children: vec![BottomWidget {
                                ratio: 1,
                                widget_type: BottomWidgetType::Mem,
                            }],
                        },
                        BottomCol {
                            ratio: 3,
                            children: vec![
                                BottomWidget {
                                    ratio: 1,
                                    widget_type: BottomWidgetType::Temp,
                                },
                                BottomWidget {
                                    ratio: 1,
                                    widget_type: BottomWidgetType::Disk,
                                },
                            ],
                        },
                    ],
                },
                BottomRow {
                    ratio: 1,
                    children: vec![
                        BottomCol {
                            ratio: 1,
                            children: vec![BottomWidget {
                                ratio: 1,
                                widget_type: BottomWidgetType::Net,
                            }],
                        },
                        BottomCol {
                            ratio: 1,
                            children: vec![BottomWidget {
                                ratio: 1,
                                widget_type: BottomWidgetType::Proc,
                            }],
                        },
                    ],
                },
            ],
        }
    }
}

/// Represents a single row in the layout.
#[derive(Debug)]
pub struct BottomRow {
    pub ratio: u64,
    pub children: Vec<BottomCol>,
}

/// Represents a single column in the layout.  We assume that even if the column
/// contains only ONE element, it is still a column (rather than either a col or
/// a widget, as per the config, for simplicity's sake).
#[derive(Debug)]
pub struct BottomCol {
    pub ratio: u64,
    pub children: Vec<BottomWidget>,
}

/// Represents a single widget + length.
#[derive(Debug)]
pub struct BottomWidget {
    pub ratio: u64,
    pub widget_type: BottomWidgetType,
}

#[derive(Debug)]
pub enum BottomWidgetType {
    Empty,
    Cpu,
    Mem,
    Net,
    Proc,
    Temp,
    Disk,
}

impl std::str::FromStr for BottomWidgetType {
    type Err = BottomError;

    fn from_str(s: &str) -> Result<Self> {
        let lower_case = s.to_lowercase();
        match lower_case.as_str() {
            "cpu" => Ok(BottomWidgetType::Cpu),
            "mem" => Ok(BottomWidgetType::Mem),
            "net" => Ok(BottomWidgetType::Net),
            "proc" => Ok(BottomWidgetType::Proc),
            "temp" => Ok(BottomWidgetType::Temp),
            "disk" => Ok(BottomWidgetType::Disk),
            "empty" => Ok(BottomWidgetType::Empty),
            _ => Err(BottomError::ConfigError(format!(
                "Invalid widget type: {}",
                s
            ))),
        }
    }
}
