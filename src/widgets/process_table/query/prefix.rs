use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter},
    time::Duration,
};

use humantime::parse_duration;
use regex::Regex;

use crate::{
    collection::processes::ProcessHarvest,
    utils::data_units::*,
    widgets::query::{
        And, ComparableQuery, NumericalQuery, Or, PrefixType, QueryComparison, QueryProcessor,
        QueryResult, StringQuery, TimeQuery, error::QueryError,
    },
};

#[inline]
fn process_prefix_units(query: &mut VecDeque<String>, value: &mut f64) {
    // If no unit, assume base.
    //
    // Furthermore, base must be PEEKED at initially, and will
    // require (likely) prefix_type specific checks
    // Lastly, if it *is* a unit, remember to POP!
    if let Some(potential_unit) = query.front() {
        if potential_unit.eq_ignore_ascii_case("tb") {
            *value *= TERA_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("tib") {
            *value *= TEBI_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("gb") {
            *value *= GIGA_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("gib") {
            *value *= GIBI_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("mb") {
            *value *= MEGA_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("mib") {
            *value *= MEBI_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("kb") {
            *value *= KILO_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("kib") {
            *value *= KIBI_LIMIT_F64;
            query.pop_front();
        } else if potential_unit.eq_ignore_ascii_case("b") {
            query.pop_front();
        }
    }
}

/// Either contains a further `Or` recursively, or a "prefix" which is a leaf that can be searched.
///
// TODO: Represent this using an enum instead or something...
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
                    _ => true, // TODO: Change prefix types to be tied to the query type so we don't have the wildcard.
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
            // Somehow we have an empty condition... oh well. Return true.
            true
        }
    }

    fn process_in_quotes(query: &mut VecDeque<String>) -> QueryResult<Self> {
        if let Some(queue_top) = query.pop_front() {
            if queue_top == "\"" {
                // This means we hit something like "". Return an empty prefix, and to deal
                // with the close quote checker, add one to the top of the
                // stack. Ugly fix but whatever.
                query.push_front("\"".to_string());

                Ok(Prefix {
                    or: None,
                    regex_prefix: Some((PrefixType::Name, StringQuery::Value(String::default()))),
                    compare_prefix: None,
                })
            } else {
                let mut intern_string = vec![queue_top];

                // TODO: I think this should consume the quote...?
                while let Some(next_str) = query.front() {
                    if next_str == "\"" {
                        break;
                    } else {
                        intern_string.push(query.pop_front().expect("we just peeked at the front"));
                    }
                }

                let quoted_string = intern_string.join(" ");

                Ok(Prefix {
                    or: None,
                    regex_prefix: Some((PrefixType::Name, StringQuery::Value(quoted_string))),
                    compare_prefix: None,
                })
            }
        } else {
            // Uh oh, there's nothing left in the stack, but we're inside quotes!
            Err(QueryError::new("Missing closing quotation"))
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

impl QueryProcessor for Prefix {
    fn process(query: &mut VecDeque<String>) -> QueryResult<Self>
    where
        Self: Sized,
    {
        if let Some(curr) = query.pop_front() {
            if curr == "(" {
                if query.is_empty() {
                    return Err(QueryError::new("Missing closing parentheses"));
                }

                let mut list_of_ors = VecDeque::new();

                while let Some(in_paren_query_top) = query.front() {
                    if in_paren_query_top != ")" {
                        list_of_ors.push_back(Or::process(query)?);
                    } else {
                        break;
                    }
                }

                // Ensure not empty
                if list_of_ors.is_empty() {
                    return Err(QueryError::new("No values within parentheses group"));
                }

                // Now convert this back to a OR...
                // TODO: This seems like a bad way to do it.
                let initial_or = Or {
                    lhs: And {
                        lhs: Prefix {
                            or: list_of_ors.pop_front().map(Box::new),
                            compare_prefix: None,
                            regex_prefix: None,
                        },
                        rhs: None,
                    },
                    rhs: None,
                };
                let returned_or = list_of_ors.into_iter().fold(initial_or, |lhs, rhs| Or {
                    lhs: And {
                        lhs: Prefix {
                            or: Some(Box::new(lhs)),
                            compare_prefix: None,
                            regex_prefix: None,
                        },
                        rhs: Some(Box::new(Prefix {
                            or: Some(Box::new(rhs)),
                            compare_prefix: None,
                            regex_prefix: None,
                        })),
                    },
                    rhs: None,
                });

                if let Some(close_paren) = query.pop_front() {
                    if close_paren == ")" {
                        return Ok(Prefix {
                            or: Some(Box::new(returned_or)),
                            regex_prefix: None,
                            compare_prefix: None,
                        });
                    } else {
                        return Err(QueryError::new("Missing closing parentheses"));
                    }
                } else {
                    return Err(QueryError::new("Missing closing parentheses"));
                }
            } else if curr == ")" {
                return Err(QueryError::new("Missing opening parentheses"));
            } else if curr == "\"" {
                // Similar to parentheses, trap and check for missing closing quotes.  Note,
                // however, that we will DIRECTLY call another process_prefix
                // call...

                let prefix = Prefix::process_in_quotes(query)?;
                if let Some(close_quote) = query.pop_front() {
                    if close_quote == "\"" {
                        return Ok(prefix);
                    } else {
                        return Err(QueryError::new("Missing closing quotation"));
                    }
                } else {
                    return Err(QueryError::new("Missing closing quotation"));
                }
            } else {
                // Get prefix type.
                let prefix_type = curr.parse::<PrefixType>()?;

                // TODO: Separate these cases here and below.
                let content = if let PrefixType::Name = prefix_type {
                    Some(curr)
                } else {
                    query.pop_front()
                };

                if let Some(content) = content {
                    match &prefix_type {
                        PrefixType::Name => {
                            return Ok(Prefix {
                                or: None,
                                regex_prefix: Some((prefix_type, StringQuery::Value(content))),
                                compare_prefix: None,
                            });
                        }
                        PrefixType::Pid | PrefixType::State | PrefixType::User => {
                            // We have to check if someone put an "="...
                            if content == "=" {
                                // Check next string if possible
                                if let Some(string_value) = query.pop_front() {
                                    // TODO: [Query] Need to consider the following cases:
                                    // - (test)
                                    // - (test
                                    // - test)
                                    // These are split into 2 to 3 different strings due to
                                    // parentheses being
                                    // delimiters in our query system.
                                    //
                                    // Do we want these to be valid?  They should, as a string,
                                    // right?

                                    // We also must check if this value is wrapped in quotes!
                                    let final_value = if string_value == "\"" {
                                        let mut intern_string = vec![];

                                        // Keep parsing until we either hit another quotation or we error.
                                        while let Some(next_string) = query.pop_front() {
                                            if next_string == "\"" {
                                                break;
                                            }

                                            intern_string.push(next_string);
                                        }

                                        intern_string.join(" ")
                                    } else {
                                        string_value
                                    };

                                    return Ok(Prefix {
                                        or: None,
                                        regex_prefix: Some((
                                            prefix_type,
                                            StringQuery::Value(final_value),
                                        )),
                                        compare_prefix: None,
                                    });
                                }
                            } else {
                                return Ok(Prefix {
                                    or: None,
                                    regex_prefix: Some((prefix_type, StringQuery::Value(content))),
                                    compare_prefix: None,
                                });
                            }
                        }
                        PrefixType::Time => {
                            let mut condition: Option<QueryComparison> = None;
                            let mut duration_string: Option<String> = None;

                            if content == "=" {
                                condition = Some(QueryComparison::Equal);
                                duration_string = query.pop_front();
                            } else if content == ">" || content == "<" {
                                if let Some(queue_next) = query.pop_front() {
                                    if queue_next == "=" {
                                        condition = Some(if content == ">" {
                                            QueryComparison::GreaterOrEqual
                                        } else {
                                            QueryComparison::LessOrEqual
                                        });
                                        duration_string = query.pop_front();
                                    } else {
                                        condition = Some(if content == ">" {
                                            QueryComparison::Greater
                                        } else {
                                            QueryComparison::Less
                                        });
                                        duration_string = Some(queue_next);
                                    }
                                } else {
                                    return Err(QueryError::missing_value());
                                }
                            }

                            if let Some(condition) = condition {
                                let duration = parse_duration(
                                    &duration_string.ok_or(QueryError::missing_value())?,
                                )
                                .map_err(|err| QueryError::new(err.to_string()))?;

                                return Ok(Prefix {
                                    or: None,
                                    regex_prefix: None,
                                    compare_prefix: Some((
                                        prefix_type,
                                        ComparableQuery::Time(TimeQuery {
                                            condition,
                                            duration,
                                        }),
                                    )),
                                });
                            }
                        }
                        _ => {
                            // Assume it's some numerical value.
                            // Now we gotta parse the content... yay.

                            let mut condition: Option<QueryComparison> = None;
                            let mut value: Option<f64> = None;

                            // TODO: Jeez, what the heck did I write here... add some tests and
                            // clean this up in the future.
                            if content == "=" {
                                condition = Some(QueryComparison::Equal);
                                if let Some(queue_next) = query.pop_front() {
                                    value = queue_next.parse::<f64>().ok();
                                } else {
                                    return Err(QueryError::missing_value());
                                }
                            } else if content == ">" || content == "<" {
                                // We also have to check if the next string is an "="...
                                if let Some(queue_next) = query.pop_front() {
                                    if queue_next == "=" {
                                        condition = Some(if content == ">" {
                                            QueryComparison::GreaterOrEqual
                                        } else {
                                            QueryComparison::LessOrEqual
                                        });
                                        if let Some(queue_next_next) = query.pop_front() {
                                            value = queue_next_next.parse::<f64>().ok();
                                        } else {
                                            return Err(QueryError::missing_value());
                                        }
                                    } else {
                                        condition = Some(if content == ">" {
                                            QueryComparison::Greater
                                        } else {
                                            QueryComparison::Less
                                        });
                                        value = queue_next.parse::<f64>().ok();
                                    }
                                } else {
                                    return Err(QueryError::missing_value());
                                }
                            }

                            if let Some(condition) = condition {
                                if let Some(read_value) = value {
                                    // Note that the values *might* have a unit or need to be parsed
                                    // differently based on the
                                    // prefix type!

                                    // TODO: Support this without spaces?

                                    let mut value = read_value;

                                    match prefix_type {
                                        PrefixType::MemBytes
                                        | PrefixType::Rps
                                        | PrefixType::Wps
                                        | PrefixType::TRead
                                        | PrefixType::TWrite => {
                                            process_prefix_units(query, &mut value);
                                        }
                                        #[cfg(feature = "gpu")]
                                        PrefixType::GMem => {
                                            process_prefix_units(query, &mut value);
                                        }
                                        _ => {}
                                    }

                                    return Ok(Prefix {
                                        or: None,
                                        regex_prefix: None,
                                        compare_prefix: Some((
                                            prefix_type,
                                            ComparableQuery::Numerical(NumericalQuery {
                                                condition,
                                                value,
                                            }),
                                        )),
                                    });
                                }
                            }
                        }
                    }
                } else {
                    return Err(QueryError::new("Missing argument for search prefix"));
                }
            }
        }

        // TODO: Give more information here (e.g. closest query?), though this is moreso meant as a fallback.
        Err(QueryError::new("Invalid query"))
    }
}
