//! How we query processes.
//!
//! Yes, this is a hand-rolled parser. I originally wrote this back in uni where writing
//! a parser was basically a thing I did every year, and parsing crate options were not
//! as good as they are now.

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

use regex::Regex;

use crate::{collection::processes::ProcessHarvest, multi_eq_ignore_ascii_case};

const DELIMITER_LIST: [char; 6] = ['=', '>', '<', '(', ')', '\"'];
const COMPARISON_LIST: [&str; 3] = [">", "=", "<"];

/// A node type that can take a query and read it, advancing the current read state
/// and returning an instance of the node.
trait QueryProcessor {
    fn process(query: &mut VecDeque<String>) -> QueryResult<Self>
    where
        Self: Sized;
}

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
        let lhs = Or::process(query)?;
        let mut list_of_ors = vec![lhs];

        while query.front().is_some() {
            list_of_ors.push(Or::process(query)?);
        }

        Ok(ProcessQuery { query: list_of_ors })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_process(name: &str) -> ProcessHarvest {
        ProcessHarvest {
            name: name.into(),
            ..Default::default()
        }
    }

    #[test]
    fn basic_query() {
        let query = parse_query("test", false, false, false).unwrap();

        let exact_match = simple_process("test");
        let contains = simple_process("test string");
        let invalid = simple_process("no");

        assert!(query.check(&exact_match, false));
        assert!(query.check(&contains, false));
        assert!(!query.check(&invalid, false));
    }

    #[test]
    fn basic_or_query() {
        let query = parse_query("a or b", false, false, false).unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let invalid = simple_process("c");

        assert!(query.check(&a, false));
        assert!(query.check(&b, false));
        assert!(!query.check(&invalid, false));
    }

    #[test]
    fn basic_and_query() {
        let query = parse_query("a and b", false, false, false).unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let c = simple_process("c");
        let a_and_b = simple_process("a and b");

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&c, false));
        assert!(query.check(&a_and_b, false));
    }

    /// Ensure that quoted keywords are treated as strings. In this case, rather than `"a" OR "b"`, it should be treated
    /// as the string `"a or b"`.
    #[test]
    fn quoted_query() {
        let query = parse_query("a \"or\" b", false, false, false).unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let or = simple_process("or");
        let valid = simple_process("a \"or\" b");
        let also_valid = simple_process("a \"or\" b \"or\" c");

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&or, false));
        assert!(query.check(&valid, false));
        assert!(query.check(&also_valid, false));
    }

    #[test]
    fn basic_cpu_query() {
        let query = parse_query("cpu > 50", false, false, false).unwrap();

        let mut over = simple_process("a");
        over.cpu_usage_percent = 60.0;

        let mut under = simple_process("a");
        under.cpu_usage_percent = 40.0;

        let mut exact = simple_process("a");
        exact.cpu_usage_percent = 50.0;

        assert!(query.check(&over, false));
        assert!(!query.check(&under, false));
        assert!(!query.check(&exact, false));
    }

    #[test]
    fn basic_mem_query() {
        let query = parse_query("memb > 1 GiB", false, false, false).unwrap();

        let mut over = simple_process("a");
        over.mem_usage = 2 * 1024 * 1024 * 1024;

        let mut under = simple_process("a");
        under.mem_usage = 0;

        let mut exact = simple_process("a");
        exact.mem_usage = 1024 * 1024 * 1024;

        assert!(query.check(&over, false));
        assert!(!query.check(&under, false));
        assert!(!query.check(&exact, false));
    }

    /// This test sees if parentheses work.
    #[test]
    fn nested_query_1() {
        let query = parse_query("(a or b) and (c or a)", false, false, false).unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let c = simple_process("c");
        let d = simple_process("d");

        assert!(query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&c, false));
        assert!(!query.check(&d, false));
    }

    /// This test sees if parentheses and mixed query types work.
    #[test]
    fn nested_query_2() {
        let query = parse_query("(cpu > 10 or cpu < 5) and (c or a)", false, false, false).unwrap();

        let mut a_valid_1 = simple_process("a");
        a_valid_1.cpu_usage_percent = 100.0;

        let mut a_valid_2 = simple_process("a");
        a_valid_2.cpu_usage_percent = 1.0;

        let mut a_invalid = simple_process("a");
        a_invalid.cpu_usage_percent = 6.0;

        let mut c = simple_process("c");
        c.cpu_usage_percent = 50.0;

        let mut b = simple_process("b");
        b.cpu_usage_percent = 50.0;

        let mut d = simple_process("d");
        d.cpu_usage_percent = 6.0;

        assert!(query.check(&a_valid_1, false));
        assert!(query.check(&a_valid_2, false));
        assert!(query.check(&c, false));

        assert!(!query.check(&a_invalid, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&d, false));
    }

    /// This test adds a further layer of nesting to consider.
    #[test]
    fn nested_query_3() {
        let query = parse_query(
            "((cpu > 10 or cpu < 5) or d) and ((c or a) or d)",
            false,
            false,
            false,
        )
        .unwrap();

        let mut a_valid_1 = simple_process("a");
        a_valid_1.cpu_usage_percent = 100.0;

        let mut a_valid_2 = simple_process("a");
        a_valid_2.cpu_usage_percent = 1.0;

        let mut a_invalid = simple_process("a");
        a_invalid.cpu_usage_percent = 6.0;

        let mut c = simple_process("c");
        c.cpu_usage_percent = 50.0;

        let mut b = simple_process("b");
        b.cpu_usage_percent = 50.0;

        let mut d = simple_process("d");
        d.cpu_usage_percent = 6.0;

        assert!(query.check(&a_valid_1, false));
        assert!(query.check(&a_valid_2, false));
        assert!(query.check(&c, false));
        assert!(query.check(&d, false));

        assert!(!query.check(&a_invalid, false));
        assert!(!query.check(&b, false));
    }
}
