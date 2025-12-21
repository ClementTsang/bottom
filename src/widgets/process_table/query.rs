mod and;
mod error;
mod or;
mod prefix;

use and::And;
use error::{QueryError, QueryResult};
use or::Or;
use prefix::Prefix;
use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter},
    time::Duration,
};

use humantime::parse_duration;
use regex::Regex;

use crate::{
    collection::processes::ProcessHarvest, multi_eq_ignore_ascii_case, utils::data_units::*,
};

const DELIMITER_LIST: [char; 6] = ['=', '>', '<', '(', ')', '\"'];
const COMPARISON_LIST: [&str; 3] = [">", "=", "<"];
const OR_LIST: [&str; 2] = ["or", "||"];
const AND_LIST: [&str; 2] = ["and", "&&"];

/// In charge of parsing the given query, case-insensitive, possibly marked
/// by a prefix. For example:
///
/// - Process names: No prefix required, can use regex, match word, or case.
///   Enclosing anything, including prefixes, in quotes, means we treat it as an
///   entire process rather than a prefix.
/// - PIDs: Use prefix `pid`, can use regex or match word.
/// - CPU: Use prefix `cpu`.
/// - MEM: Use prefix `mem`.
/// - STATE: Use prefix `state`.
/// - USER: Use prefix `user`.
/// - Read/s: Use prefix `r`.
/// - Write/s: Use prefix `w`.
/// - Total read: Use prefix `read`.
/// - Total write: Use prefix `write`.
///
/// For queries, whitespaces are our delimiters.  We will merge together any
/// adjacent non-prefixed or quoted elements after splitting to treat as process
/// names. Furthermore, we want to support boolean joiners like AND and OR, and
/// brackets.
pub(crate) fn parse_query(
    search_query: &str, is_searching_whole_word: bool, is_ignoring_case: bool,
    is_searching_with_regex: bool,
) -> QueryResult<ProcessQuery> {
    fn process_string_to_filter(query: &mut VecDeque<String>) -> QueryResult<ProcessQuery> {
        let lhs = process_or(query)?;
        let mut list_of_ors = vec![lhs];

        while query.front().is_some() {
            list_of_ors.push(process_or(query)?);
        }

        Ok(ProcessQuery { query: list_of_ors })
    }

    fn process_or(query: &mut VecDeque<String>) -> QueryResult<Or> {
        let mut lhs = process_and(query)?;
        let mut rhs: Option<Box<And>> = None;

        while let Some(queue_top) = query.front() {
            let current_lowercase = queue_top.to_lowercase();
            if OR_LIST.contains(&current_lowercase.as_str()) {
                query.pop_front();
                rhs = Some(Box::new(process_and(query)?));

                if let Some(queue_next) = query.front() {
                    if OR_LIST.contains(&queue_next.to_lowercase().as_str()) {
                        // Must merge LHS and RHS
                        lhs = And {
                            lhs: Prefix {
                                or: Some(Box::new(Or { lhs, rhs })),
                                regex_prefix: None,
                                compare_prefix: None,
                            },
                            rhs: None,
                        };
                        rhs = None;
                    }
                } else {
                    break;
                }
            } else if COMPARISON_LIST.contains(&current_lowercase.as_str()) {
                return Err(QueryError::new("Comparison not valid here"));
            } else {
                break;
            }
        }

        Ok(Or { lhs, rhs })
    }

    fn process_and(query: &mut VecDeque<String>) -> QueryResult<And> {
        let mut lhs = process_prefix(query, false)?;
        let mut rhs: Option<Box<Prefix>> = None;

        while let Some(queue_top) = query.front() {
            let current_lowercase = queue_top.to_lowercase();
            if AND_LIST.contains(&current_lowercase.as_str()) {
                query.pop_front();

                rhs = Some(Box::new(process_prefix(query, false)?));

                if let Some(next_queue_top) = query.front() {
                    if AND_LIST.contains(&next_queue_top.to_lowercase().as_str()) {
                        // Must merge LHS and RHS
                        lhs = Prefix {
                            or: Some(Box::new(Or {
                                lhs: And { lhs, rhs },
                                rhs: None,
                            })),
                            regex_prefix: None,
                            compare_prefix: None,
                        };
                        rhs = None;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if COMPARISON_LIST.contains(&current_lowercase.as_str()) {
                return Err(QueryError::new("Comparison not valid here"));
            } else {
                break;
            }
        }

        Ok(And { lhs, rhs })
    }

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

    fn process_prefix(query: &mut VecDeque<String>, inside_quotation: bool) -> QueryResult<Prefix> {
        if let Some(queue_top) = query.pop_front() {
            if inside_quotation {
                if queue_top == "\"" {
                    // This means we hit something like "".  Return an empty prefix, and to deal
                    // with the close quote checker, add one to the top of the
                    // stack.  Ugly fix but whatever.
                    query.push_front("\"".to_string());
                    return Ok(Prefix {
                        or: None,
                        regex_prefix: Some((
                            PrefixType::Name,
                            StringQuery::Value(String::default()),
                        )),
                        compare_prefix: None,
                    });
                } else {
                    let mut quoted_string = queue_top;
                    while let Some(next_str) = query.front() {
                        if next_str == "\"" {
                            // Stop!
                            break;
                        } else {
                            quoted_string.push_str(next_str);
                            query.pop_front();
                        }
                    }
                    return Ok(Prefix {
                        or: None,
                        regex_prefix: Some((PrefixType::Name, StringQuery::Value(quoted_string))),
                        compare_prefix: None,
                    });
                }
            } else if queue_top == "(" {
                if query.is_empty() {
                    return Err(QueryError::new("Missing closing parentheses"));
                }

                let mut list_of_ors = VecDeque::new();

                while let Some(in_paren_query_top) = query.front() {
                    if in_paren_query_top != ")" {
                        list_of_ors.push_back(process_or(query)?);
                    } else {
                        break;
                    }
                }

                // Ensure not empty
                if list_of_ors.is_empty() {
                    return Err(QueryError::new("No values within parentheses group"));
                }

                // Now convert this back to a OR...
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
            } else if queue_top == ")" {
                return Err(QueryError::new("Missing opening parentheses"));
            } else if queue_top == "\"" {
                // Similar to parentheses, trap and check for missing closing quotes.  Note,
                // however, that we will DIRECTLY call another process_prefix
                // call...

                let prefix = process_prefix(query, true)?;
                if let Some(close_paren) = query.pop_front() {
                    if close_paren == "\"" {
                        return Ok(prefix);
                    } else {
                        return Err(QueryError::new("Missing closing quotation"));
                    }
                } else {
                    return Err(QueryError::new("Missing closing quotation"));
                }
            } else {
                // Get prefix type.
                let prefix_type = queue_top.parse::<PrefixType>()?;
                let content = if let PrefixType::Name = prefix_type {
                    Some(queue_top)
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
                                if let Some(queue_next) = query.pop_front() {
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

                                    return Ok(Prefix {
                                        or: None,
                                        regex_prefix: Some((
                                            prefix_type,
                                            StringQuery::Value(queue_next),
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
        } else if inside_quotation {
            // Uh oh, it's empty with quotes!
            return Err(QueryError::new("Missing closing quotation"));
        }

        Err(QueryError::new("Invalid query"))
    }

    let mut split_query = VecDeque::new();

    search_query.split_whitespace().for_each(|s| {
        // From https://stackoverflow.com/a/56923739 in order to get a split, but include the parentheses
        let mut last = 0;
        for (index, matched) in s.match_indices(|x| DELIMITER_LIST.contains(&x)) {
            if last != index {
                split_query.push_back(s[last..index].to_owned());
            }
            split_query.push_back(matched.to_owned());
            last = index + matched.len();
        }
        if last < s.len() {
            split_query.push_back(s[last..].to_owned());
        }
    });

    let mut process_filter = process_string_to_filter(&mut split_query)?;
    process_filter.process_regexes(
        is_searching_whole_word,
        is_ignoring_case,
        is_searching_with_regex,
    )?;

    Ok(process_filter)
}

pub struct ProcessQuery {
    /// Remember, AND > OR, but AND must come after OR when we parse.
    query: Vec<Or>,
}

impl ProcessQuery {
    fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> QueryResult<()> {
        for or in &mut self.query {
            or.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            )?;
        }

        Ok(())
    }

    pub(crate) fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        self.query
            .iter()
            .all(|ok| ok.check(process, is_using_command))
    }
}

impl Debug for ProcessQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.query))
    }
}

#[derive(Debug)]
enum PrefixType {
    Pid,
    PCpu,
    MemBytes,
    PMem,
    Rps,
    Wps,
    TRead,
    TWrite,
    Name,
    State,
    User,
    Time,
    #[cfg(unix)]
    Nice,
    Priority,
    #[cfg(feature = "gpu")]
    PGpu,
    #[cfg(feature = "gpu")]
    GMem,
    #[cfg(feature = "gpu")]
    PGMem,
    __Nonexhaustive,
}

impl std::str::FromStr for PrefixType {
    type Err = QueryError;

    fn from_str(s: &str) -> QueryResult<Self> {
        use PrefixType::*;

        // TODO: Didn't add mem_bytes, total_read, and total_write
        // for now as it causes help to be clogged.

        let mut result = Name;
        if multi_eq_ignore_ascii_case!(s, "cpu" | "cpu%") {
            result = PCpu;
        } else if multi_eq_ignore_ascii_case!(s, "mem" | "mem%") {
            result = PMem;
        } else if multi_eq_ignore_ascii_case!(s, "memb") {
            result = MemBytes;
        } else if multi_eq_ignore_ascii_case!(s, "read" | "r/s" | "rps") {
            result = Rps;
        } else if multi_eq_ignore_ascii_case!(s, "write" | "w/s" | "wps") {
            result = Wps;
        } else if multi_eq_ignore_ascii_case!(s, "tread" | "t.read") {
            result = TRead;
        } else if multi_eq_ignore_ascii_case!(s, "twrite" | "t.write") {
            result = TWrite;
        } else if multi_eq_ignore_ascii_case!(s, "pid") {
            result = Pid;
        } else if multi_eq_ignore_ascii_case!(s, "state") {
            result = State;
        } else if multi_eq_ignore_ascii_case!(s, "user") {
            result = User;
        } else if multi_eq_ignore_ascii_case!(s, "time") {
            result = Time;
        } else if multi_eq_ignore_ascii_case!(s, "nice") {
            #[cfg(unix)]
            {
                result = Nice;
            }
        } else if multi_eq_ignore_ascii_case!(s, "priority") {
            result = Priority;
        }
        #[cfg(feature = "gpu")]
        {
            if multi_eq_ignore_ascii_case!(s, "gmem") {
                result = GMem;
            } else if multi_eq_ignore_ascii_case!(s, "gmem%") {
                result = PGMem;
            } else if multi_eq_ignore_ascii_case!(s, "gpu%") {
                result = PGpu;
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
enum QueryComparison {
    Equal,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

#[derive(Debug)]
enum StringQuery {
    Value(String),
    Regex(Regex),
}

#[derive(Debug)]
enum ComparableQuery {
    Numerical(NumericalQuery),
    Time(TimeQuery),
}

#[derive(Debug)]
struct NumericalQuery {
    condition: QueryComparison,
    value: f64,
}

#[derive(Debug)]
struct TimeQuery {
    condition: QueryComparison,
    duration: Duration,
}
