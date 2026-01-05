//! Code related to attributes, which should be "searchable" leaf nodes.

use crate::collection::processes::ProcessHarvest;
use crate::widgets::query::error::{QueryError, QueryResult};
use crate::widgets::query::{NumericalQuery, PrefixType, QueryOptions, TimeQuery, new_regex};
use regex::Regex;

/// An attribute (leaf node) for a process.
#[derive(Debug)]
pub(super) enum ProcessAttribute {
    /// Temp hack to allow for "empty" attributes.
    Empty,
    Pid(Regex),
    CpuPercentage(NumericalQuery),
    MemBytes(NumericalQuery),
    MemPercentage(NumericalQuery),
    ReadPerSecond(NumericalQuery),
    WritePerSecond(NumericalQuery),
    TotalRead(NumericalQuery),
    TotalWrite(NumericalQuery),
    /// Note this is an "untagged" attribute (e.g. "btm", "firefox").
    Name(Regex),
    State(Regex),
    User(Regex),
    Time(TimeQuery),
    #[cfg(unix)]
    Nice(NumericalQuery),
    Priority(NumericalQuery),
    #[cfg(feature = "gpu")]
    GpuPercentage(NumericalQuery),
    #[cfg(feature = "gpu")]
    GpuMemoryPercentage(NumericalQuery),
    #[cfg(feature = "gpu")]
    GpuMemoryBytes(NumericalQuery),
}

impl ProcessAttribute {
    pub(super) fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        match self {
            ProcessAttribute::Empty => true,
            ProcessAttribute::Pid(re) => re.is_match(process.pid.to_string().as_str()),
            ProcessAttribute::CpuPercentage(cmp) => cmp.check(process.cpu_usage_percent),
            ProcessAttribute::MemBytes(cmp) => cmp.check(process.mem_usage as f64),
            ProcessAttribute::MemPercentage(cmp) => cmp.check(process.mem_usage_percent),
            ProcessAttribute::ReadPerSecond(cmp) => cmp.check(process.read_per_sec as f64),
            ProcessAttribute::WritePerSecond(cmp) => cmp.check(process.write_per_sec as f64),
            ProcessAttribute::TotalRead(cmp) => cmp.check(process.total_read as f64),
            ProcessAttribute::TotalWrite(cmp) => cmp.check(process.total_write as f64),
            ProcessAttribute::Name(re) => re.is_match(if is_using_command {
                process.command.as_str()
            } else {
                process.name.as_str()
            }),
            ProcessAttribute::State(re) => re.is_match(process.process_state.0),
            ProcessAttribute::User(re) => match process.user.as_ref() {
                Some(user) => re.is_match(user),
                None => re.is_match("N/A"),
            },
            ProcessAttribute::Time(time) => time.check(process.time),
            // TODO: It's a bit silly for some of these, like nice/priority, where it's casted to an f64.
            #[cfg(unix)]
            ProcessAttribute::Nice(cmp) => cmp.check(process.nice as f64),
            ProcessAttribute::Priority(cmp) => cmp.check(process.priority as f64),
            #[cfg(feature = "gpu")]
            ProcessAttribute::GpuPercentage(cmp) => cmp.check(process.gpu_util as f64),
            #[cfg(feature = "gpu")]
            ProcessAttribute::GpuMemoryPercentage(cmp) => cmp.check(process.gpu_mem_percent as f64),
            #[cfg(feature = "gpu")]
            ProcessAttribute::GpuMemoryBytes(cmp) => cmp.check(process.gpu_mem as f64),
        }
    }
}

/// Given a string prefix type, obtain the appropriate [`ProcessAttribute`].
pub(super) fn new_string_attribute(
    prefix_type: PrefixType, base: &str, regex_options: &QueryOptions,
) -> QueryResult<ProcessAttribute> {
    match prefix_type {
        PrefixType::Pid | PrefixType::Name | PrefixType::State | PrefixType::User => {
            let re = new_regex(base, regex_options)?;

            match prefix_type {
                PrefixType::Pid => Ok(ProcessAttribute::Pid(re)),
                PrefixType::Name => Ok(ProcessAttribute::Name(re)),
                PrefixType::State => Ok(ProcessAttribute::State(re)),
                PrefixType::User => Ok(ProcessAttribute::User(re)),
                _ => unreachable!(),
            }
        }
        _ => Err(QueryError::new(format!(
            "process attribute type {prefix_type:?} is not a supported string attribute"
        ))),
    }
}

/// Given a time prefix type, obtain the appropriate [`ProcessAttribute`].
pub(super) fn new_time_attribute(
    prefix_type: PrefixType, query: TimeQuery,
) -> QueryResult<ProcessAttribute> {
    match prefix_type {
        PrefixType::Time => Ok(ProcessAttribute::Time(query)),
        _ => Err(QueryError::new(format!(
            "process attribute type {prefix_type:?} is not a supported time attribute"
        ))),
    }
}

/// Given a numerical prefix type, obtain the appropriate [`ProcessAttribute`].
pub(super) fn new_numerical_attribute(
    prefix_type: PrefixType, query: NumericalQuery,
) -> QueryResult<ProcessAttribute> {
    match prefix_type {
        PrefixType::CpuPercentage => Ok(ProcessAttribute::CpuPercentage(query)),
        PrefixType::MemBytes => Ok(ProcessAttribute::MemBytes(query)),
        PrefixType::MemPercentage => Ok(ProcessAttribute::MemPercentage(query)),
        PrefixType::ReadPerSecond => Ok(ProcessAttribute::ReadPerSecond(query)),
        PrefixType::WritePerSecond => Ok(ProcessAttribute::WritePerSecond(query)),
        PrefixType::TotalRead => Ok(ProcessAttribute::TotalRead(query)),
        PrefixType::TotalWrite => Ok(ProcessAttribute::TotalWrite(query)),
        #[cfg(unix)]
        PrefixType::Nice => Ok(ProcessAttribute::Nice(query)),
        PrefixType::Priority => Ok(ProcessAttribute::Priority(query)),
        #[cfg(feature = "gpu")]
        PrefixType::GpuPercentage => Ok(ProcessAttribute::GpuPercentage(query)),
        #[cfg(feature = "gpu")]
        PrefixType::GpuMemoryBytes => Ok(ProcessAttribute::GpuMemoryBytes(query)),
        #[cfg(feature = "gpu")]
        PrefixType::GpuMemoryPercentage => Ok(ProcessAttribute::GpuMemoryPercentage(query)),
        _ => Err(QueryError::new(format!(
            "process attribute type {prefix_type:?} is not a supported numerical attribute"
        ))),
    }
}
