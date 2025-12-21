use std::{
    fmt::{Debug, Formatter},
    time::Duration,
};

use regex::Regex;

use crate::{
    collection::processes::ProcessHarvest,
    widgets::query::{ComparableQuery, Or, PrefixType, QueryComparison, QueryResult, StringQuery},
};

// TODO: This is also jank and could be better represented. Add tests, then
// clean up!
#[derive(Default)]
pub(super) struct Prefix {
    pub(super) or: Option<Box<Or>>,
    pub(super) regex_prefix: Option<(PrefixType, StringQuery)>,
    pub(super) compare_prefix: Option<(PrefixType, ComparableQuery)>,
}

impl Prefix {
    pub(super) fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> QueryResult<()> {
        if let Some(or) = &mut self.or {
            return or.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            );
        } else if let Some((
            PrefixType::Pid | PrefixType::Name | PrefixType::State | PrefixType::User,
            StringQuery::Value(regex_string),
        )) = &mut self.regex_prefix
        {
            let escaped_regex: String;
            let final_regex_string = &format!(
                "{}{}{}{}",
                if is_searching_whole_word { "^" } else { "" },
                if is_ignoring_case { "(?i)" } else { "" },
                if !is_searching_with_regex {
                    escaped_regex = regex::escape(regex_string);
                    &escaped_regex
                } else {
                    regex_string
                },
                if is_searching_whole_word { "$" } else { "" },
            );

            let taken_pwc = self.regex_prefix.take();
            if let Some((taken_pt, _)) = taken_pwc {
                self.regex_prefix = Some((
                    taken_pt,
                    StringQuery::Regex(Regex::new(final_regex_string)?),
                ));
            }
        }

        Ok(())
    }

    pub(super) fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        fn matches_condition<I: Into<f64>, J: Into<f64>>(
            condition: &QueryComparison, lhs: I, rhs: J,
        ) -> bool {
            let lhs: f64 = lhs.into();
            let rhs: f64 = rhs.into();

            match condition {
                QueryComparison::Equal => (lhs - rhs).abs() < f64::EPSILON,
                QueryComparison::Less => lhs < rhs,
                QueryComparison::Greater => lhs > rhs,
                QueryComparison::LessOrEqual => lhs <= rhs,
                QueryComparison::GreaterOrEqual => lhs >= rhs,
            }
        }

        fn matches_duration(condition: &QueryComparison, lhs: Duration, rhs: Duration) -> bool {
            match condition {
                QueryComparison::Equal => lhs == rhs,
                QueryComparison::Less => lhs < rhs,
                QueryComparison::Greater => lhs > rhs,
                QueryComparison::LessOrEqual => lhs <= rhs,
                QueryComparison::GreaterOrEqual => lhs >= rhs,
            }
        }

        if let Some(and) = &self.or {
            and.check(process, is_using_command)
        } else if let Some((prefix_type, query_content)) = &self.regex_prefix {
            if let StringQuery::Regex(r) = query_content {
                match prefix_type {
                    PrefixType::Name => r.is_match(if is_using_command {
                        process.command.as_str()
                    } else {
                        process.name.as_str()
                    }),
                    PrefixType::Pid => r.is_match(process.pid.to_string().as_str()),
                    PrefixType::State => r.is_match(process.process_state.0),
                    PrefixType::User => match process.user.as_ref() {
                        Some(user) => r.is_match(user),
                        None => r.is_match("N/A"),
                    },
                    _ => true,
                }
            } else {
                true
            }
        } else if let Some((prefix_type, comparable_query)) = &self.compare_prefix {
            match comparable_query {
                ComparableQuery::Numerical(numerical_query) => match prefix_type {
                    PrefixType::PCpu => matches_condition(
                        &numerical_query.condition,
                        process.cpu_usage_percent,
                        numerical_query.value,
                    ),
                    PrefixType::PMem => matches_condition(
                        &numerical_query.condition,
                        process.mem_usage_percent,
                        numerical_query.value,
                    ),
                    PrefixType::MemBytes => matches_condition(
                        &numerical_query.condition,
                        process.mem_usage as f64,
                        numerical_query.value,
                    ),
                    PrefixType::Rps => matches_condition(
                        &numerical_query.condition,
                        process.read_per_sec as f64,
                        numerical_query.value,
                    ),
                    PrefixType::Wps => matches_condition(
                        &numerical_query.condition,
                        process.write_per_sec as f64,
                        numerical_query.value,
                    ),
                    PrefixType::TRead => matches_condition(
                        &numerical_query.condition,
                        process.total_read as f64,
                        numerical_query.value,
                    ),
                    PrefixType::TWrite => matches_condition(
                        &numerical_query.condition,
                        process.total_write as f64,
                        numerical_query.value,
                    ),
                    #[cfg(feature = "gpu")]
                    PrefixType::PGpu => matches_condition(
                        &numerical_query.condition,
                        process.gpu_util,
                        numerical_query.value,
                    ),
                    #[cfg(feature = "gpu")]
                    PrefixType::GMem => matches_condition(
                        &numerical_query.condition,
                        process.gpu_mem as f64,
                        numerical_query.value,
                    ),
                    #[cfg(feature = "gpu")]
                    PrefixType::PGMem => matches_condition(
                        &numerical_query.condition,
                        process.gpu_mem_percent,
                        numerical_query.value,
                    ),
                    #[cfg(unix)]
                    PrefixType::Nice => matches_condition(
                        &numerical_query.condition,
                        process.nice,
                        numerical_query.value,
                    ),
                    PrefixType::Priority => matches_condition(
                        &numerical_query.condition,
                        process.priority,
                        numerical_query.value,
                    ),
                    _ => true,
                },
                ComparableQuery::Time(time_query) => match prefix_type {
                    PrefixType::Time => {
                        matches_duration(&time_query.condition, process.time, time_query.duration)
                    }
                    _ => true,
                },
            }
        } else {
            // Somehow we have an empty condition... oh well.  Return true.
            true
        }
    }
}

impl Debug for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(or) = &self.or {
            f.write_fmt(format_args!("{or:?}"))
        } else if let Some(regex_prefix) = &self.regex_prefix {
            f.write_fmt(format_args!("{regex_prefix:?}"))
        } else if let Some(compare_prefix) = &self.compare_prefix {
            f.write_fmt(format_args!("{compare_prefix:?}"))
        } else {
            f.write_str("")
        }
    }
}
