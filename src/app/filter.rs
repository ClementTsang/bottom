#[derive(Debug, Clone)]
pub struct Filter {
    pub is_list_ignored: bool,
    pub list: Vec<regex::Regex>,
}

/// For filtering out information
#[derive(Debug, Clone)]
pub struct DataFilters {
    pub disk_filter: Option<Filter>,
    pub mount_filter: Option<Filter>,
    pub temp_filter: Option<Filter>,
    pub net_filter: Option<Filter>,
}
