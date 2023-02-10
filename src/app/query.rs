use std::fmt::Debug;
use std::{borrow::Cow, collections::VecDeque};

use super::data_harvester::processes::ProcessHarvest;
use crate::utils::error::{
    BottomError::{self, QueryError},
    Result,
};

const DELIMITER_LIST: [char; 6] = ['=', '>', '<', '(', ')', '\"'];
const COMPARISON_LIST: [&str; 3] = [">", "=", "<"];
const OR_LIST: [&str; 2] = ["or", "||"];
const AND_LIST: [&str; 2] = ["and", "&&"];

/// In charge of parsing the given query.
/// We are defining the following language for a query (case-insensitive prefixes):
///
/// - Process names: No prefix required, can use regex, match word, or case.
///   Enclosing anything, including prefixes, in quotes, means we treat it as an entire process
///   rather than a prefix.
/// - PIDs: Use prefix `pid`, can use regex or match word (case is irrelevant).
/// - CPU: Use prefix `cpu`, cannot use r/m/c (regex, match word, case).  Can compare.
/// - MEM: Use prefix `mem`, cannot use r/m/c.  Can compare.
/// - STATE: Use prefix `state`, can use regex, match word, or case.
/// - USER: Use prefix `user`, can use regex, match word, or case.
/// - Read/s: Use prefix `r`.  Can compare.
/// - Write/s: Use prefix `w`.  Can compare.
/// - Total read: Use prefix `read`.  Can compare.
/// - Total write: Use prefix `write`.  Can compare.
///
/// For queries, whitespaces are our delimiters.  We will merge together any adjacent non-prefixed
/// or quoted elements after splitting to treat as process names.
/// Furthermore, we want to support boolean joiners like AND and OR, and brackets.
pub fn parse_query(
    search_query: &str, is_searching_whole_word: bool, is_ignoring_case: bool,
    is_searching_with_regex: bool,
) -> Result<Query> {
    fn process_string_to_filter(query: &mut VecDeque<String>) -> Result<Query> {
        let lhs = process_or(query)?;
        let mut list_of_ors = vec![lhs];

        while query.front().is_some() {
            list_of_ors.push(process_or(query)?);
        }

        Ok(Query { query: list_of_ors })
    }

    fn process_or(query: &mut VecDeque<String>) -> Result<Or> {
        let mut lhs = process_and(query)?;
        let mut rhs: Option<Box<And>> = None;

        while let Some(queue_top) = query.front() {
            // debug!("OR QT: {:?}", queue_top);
            if OR_LIST.contains(&queue_top.to_lowercase().as_str()) {
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
            } else if COMPARISON_LIST.contains(&queue_top.to_lowercase().as_str()) {
                return Err(QueryError(Cow::Borrowed("Comparison not valid here")));
            } else {
                break;
            }
        }

        Ok(Or { lhs, rhs })
    }

    fn process_and(query: &mut VecDeque<String>) -> Result<And> {
        let mut lhs = process_prefix(query, false)?;
        let mut rhs: Option<Box<Prefix>> = None;

        while let Some(queue_top) = query.front() {
            // debug!("AND QT: {:?}", queue_top);
            if AND_LIST.contains(&queue_top.to_lowercase().as_str()) {
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
            } else if COMPARISON_LIST.contains(&queue_top.to_lowercase().as_str()) {
                return Err(QueryError(Cow::Borrowed("Comparison not valid here")));
            } else {
                break;
            }
        }

        Ok(And { lhs, rhs })
    }

    fn process_prefix(query: &mut VecDeque<String>, inside_quotation: bool) -> Result<Prefix> {
        if let Some(queue_top) = query.pop_front() {
            if inside_quotation {
                if queue_top == "\"" {
                    // This means we hit something like "".  Return an empty prefix, and to deal with
                    // the close quote checker, add one to the top of the stack.  Ugly fix but whatever.
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
                    return Err(QueryError(Cow::Borrowed("Missing closing parentheses")));
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
                    return Err(QueryError("No values within parentheses group".into()));
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
                        return Err(QueryError("Missing closing parentheses".into()));
                    }
                } else {
                    return Err(QueryError("Missing closing parentheses".into()));
                }
            } else if queue_top == ")" {
                return Err(QueryError("Missing opening parentheses".into()));
            } else if queue_top == "\"" {
                // Similar to parentheses, trap and check for missing closing quotes.  Note, however, that we
                // will DIRECTLY call another process_prefix call...

                let prefix = process_prefix(query, true)?;
                if let Some(close_paren) = query.pop_front() {
                    if close_paren == "\"" {
                        return Ok(prefix);
                    } else {
                        return Err(QueryError("Missing closing quotation".into()));
                    }
                } else {
                    return Err(QueryError("Missing closing quotation".into()));
                }
            } else {
                //  Get prefix type...
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
                            })
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
                                    // These are split into 2 to 3 different strings due to parentheses being
                                    // delimiters in our query system.
                                    //
                                    // Do we want these to be valid?  They should, as a string, right?

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
                        _ => {
                            // Now we gotta parse the content... yay.

                            let mut condition: Option<QueryComparison> = None;
                            let mut value: Option<f64> = None;

                            if content == "=" {
                                condition = Some(QueryComparison::Equal);
                                if let Some(queue_next) = query.pop_front() {
                                    value = queue_next.parse::<f64>().ok();
                                } else {
                                    return Err(QueryError("Missing value".into()));
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
                                            return Err(QueryError("Missing value".into()));
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
                                    return Err(QueryError("Missing value".into()));
                                }
                            }

                            if let Some(condition) = condition {
                                if let Some(read_value) = value {
                                    // Now we want to check one last thing - is there a unit?
                                    // If no unit, assume base.
                                    // Furthermore, base must be PEEKED at initially, and will
                                    // require (likely) prefix_type specific checks
                                    // Lastly, if it *is* a unit, remember to POP!

                                    let mut value = read_value;

                                    match prefix_type {
                                        PrefixType::MemBytes
                                        | PrefixType::Rps
                                        | PrefixType::Wps
                                        | PrefixType::TRead
                                        | PrefixType::TWrite => {
                                            if let Some(potential_unit) = query.front() {
                                                match potential_unit.to_lowercase().as_str() {
                                                    "tb" => {
                                                        value *= 1_000_000_000_000.0;
                                                        query.pop_front();
                                                    }
                                                    "tib" => {
                                                        value *= 1_099_511_627_776.0;
                                                        query.pop_front();
                                                    }
                                                    "gb" => {
                                                        value *= 1_000_000_000.0;
                                                        query.pop_front();
                                                    }
                                                    "gib" => {
                                                        value *= 1_073_741_824.0;
                                                        query.pop_front();
                                                    }
                                                    "mb" => {
                                                        value *= 1_000_000.0;
                                                        query.pop_front();
                                                    }
                                                    "mib" => {
                                                        value *= 1_048_576.0;
                                                        query.pop_front();
                                                    }
                                                    "kb" => {
                                                        value *= 1000.0;
                                                        query.pop_front();
                                                    }
                                                    "kib" => {
                                                        value *= 1024.0;
                                                        query.pop_front();
                                                    }
                                                    "b" => {
                                                        // Just gotta pop.
                                                        query.pop_front();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }

                                    return Ok(Prefix {
                                        or: None,
                                        regex_prefix: None,
                                        compare_prefix: Some((
                                            prefix_type,
                                            NumericalQuery { condition, value },
                                        )),
                                    });
                                }
                            }
                        }
                    }
                } else {
                    return Err(QueryError("Missing argument for search prefix".into()));
                }
            }
        } else if inside_quotation {
            // Uh oh, it's empty with quotes!
            return Err(QueryError("Missing closing quotation".into()));
        }

        Err(QueryError("Invalid query".into()))
    }

    let mut split_query = VecDeque::new();

    search_query.split_whitespace().for_each(|s| {
        // From https://stackoverflow.com/a/56923739 in order to get a split but include the parentheses
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

pub struct Query {
    /// Remember, AND > OR, but AND must come after OR when we parse.
    pub query: Vec<Or>,
}

impl Query {
    pub fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> Result<()> {
        for or in &mut self.query {
            or.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            )?;
        }

        Ok(())
    }

    pub fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        self.query
            .iter()
            .all(|ok| ok.check(process, is_using_command))
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.query))
    }
}

#[derive(Default)]
pub struct Or {
    pub lhs: And,
    pub rhs: Option<Box<And>>,
}

impl Or {
    pub fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> Result<()> {
        self.lhs.process_regexes(
            is_searching_whole_word,
            is_ignoring_case,
            is_searching_with_regex,
        )?;
        if let Some(rhs) = &mut self.rhs {
            rhs.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            )?;
        }

        Ok(())
    }

    pub fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        if let Some(rhs) = &self.rhs {
            self.lhs.check(process, is_using_command) || rhs.check(process, is_using_command)
        } else {
            self.lhs.check(process, is_using_command)
        }
    }
}

impl Debug for Or {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.rhs {
            Some(rhs) => f.write_fmt(format_args!("({:?} OR {:?})", self.lhs, rhs)),
            None => f.write_fmt(format_args!("{:?}", self.lhs)),
        }
    }
}

#[derive(Default)]
pub struct And {
    pub lhs: Prefix,
    pub rhs: Option<Box<Prefix>>,
}

impl And {
    pub fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> Result<()> {
        self.lhs.process_regexes(
            is_searching_whole_word,
            is_ignoring_case,
            is_searching_with_regex,
        )?;
        if let Some(rhs) = &mut self.rhs {
            rhs.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            )?;
        }

        Ok(())
    }

    pub fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        if let Some(rhs) = &self.rhs {
            self.lhs.check(process, is_using_command) && rhs.check(process, is_using_command)
        } else {
            self.lhs.check(process, is_using_command)
        }
    }
}

impl Debug for And {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.rhs {
            Some(rhs) => f.write_fmt(format_args!("({:?} AND {:?})", self.lhs, rhs)),
            None => f.write_fmt(format_args!("{:?}", self.lhs)),
        }
    }
}

#[derive(Debug)]
pub enum PrefixType {
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
    __Nonexhaustive,
}

impl std::str::FromStr for PrefixType {
    type Err = BottomError;

    fn from_str(s: &str) -> Result<Self> {
        use PrefixType::*;

        let lower_case = s.to_lowercase();
        // Didn't add mem_bytes, total_read, and total_write
        // for now as it causes help to be clogged.
        match lower_case.as_str() {
            "cpu" | "cpu%" => Ok(PCpu),
            "mem" | "mem%" => Ok(PMem),
            "memb" => Ok(MemBytes),
            "read" | "r/s" => Ok(Rps),
            "write" | "w/s" => Ok(Wps),
            "tread" | "t.read" => Ok(TRead),
            "twrite" | "t.write" => Ok(TWrite),
            "pid" => Ok(Pid),
            "state" => Ok(State),
            "user" => Ok(User),
            _ => Ok(Name),
        }
    }
}

#[derive(Default)]
pub struct Prefix {
    pub or: Option<Box<Or>>,
    pub regex_prefix: Option<(PrefixType, StringQuery)>,
    pub compare_prefix: Option<(PrefixType, NumericalQuery)>,
}

impl Prefix {
    pub fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> Result<()> {
        if let Some(or) = &mut self.or {
            return or.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            );
        } else if let Some((prefix_type, StringQuery::Value(regex_string))) = &mut self.regex_prefix
        {
            match prefix_type {
                PrefixType::Pid | PrefixType::Name | PrefixType::State | PrefixType::User => {
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
                            StringQuery::Regex(regex::Regex::new(final_regex_string)?),
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        fn matches_condition(condition: &QueryComparison, lhs: f64, rhs: f64) -> bool {
            match condition {
                QueryComparison::Equal => (lhs - rhs).abs() < std::f64::EPSILON,
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
                    PrefixType::State => r.is_match(process.process_state.0.as_str()),
                    PrefixType::User => r.is_match(process.user.as_ref()),
                    _ => true,
                }
            } else {
                true
            }
        } else if let Some((prefix_type, numerical_query)) = &self.compare_prefix {
            match prefix_type {
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
                    process.mem_usage_bytes as f64,
                    numerical_query.value,
                ),
                PrefixType::Rps => matches_condition(
                    &numerical_query.condition,
                    process.read_bytes_per_sec as f64,
                    numerical_query.value,
                ),
                PrefixType::Wps => matches_condition(
                    &numerical_query.condition,
                    process.write_bytes_per_sec as f64,
                    numerical_query.value,
                ),
                PrefixType::TRead => matches_condition(
                    &numerical_query.condition,
                    process.total_read_bytes as f64,
                    numerical_query.value,
                ),
                PrefixType::TWrite => matches_condition(
                    &numerical_query.condition,
                    process.total_write_bytes as f64,
                    numerical_query.value,
                ),
                _ => true,
            }
        } else {
            // Somehow we have an empty condition... oh well.  Return true.
            true
        }
    }
}

impl Debug for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(or) = &self.or {
            f.write_fmt(format_args!("{:?}", or))
        } else if let Some(regex_prefix) = &self.regex_prefix {
            f.write_fmt(format_args!("{:?}", regex_prefix))
        } else if let Some(compare_prefix) = &self.compare_prefix {
            f.write_fmt(format_args!("{:?}", compare_prefix))
        } else {
            f.write_fmt(format_args!(""))
        }
    }
}

#[derive(Debug)]
pub enum QueryComparison {
    Equal,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

#[derive(Debug)]
pub enum StringQuery {
    Value(String),
    Regex(regex::Regex),
}

#[derive(Debug)]
pub struct NumericalQuery {
    pub condition: QueryComparison,
    pub value: f64,
}
