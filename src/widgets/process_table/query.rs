//! How we query processes.
//!
//! Yes, this is a hand-rolled parser. I originally wrote this back in uni where writing
//! a parser was basically a thing I did every year, and parsing crate options were not
//! as good as they are now. This will be rewritten as time goes on, though.

mod and;
mod attribute;
mod error;
mod or;
mod prefix;

use and::And;
use attribute::ProcessAttribute;
use error::{QueryError, QueryResult};
use or::Or;
use prefix::Prefix;
use std::{collections::VecDeque, time::Duration};

use regex::Regex;

use crate::{collection::processes::ProcessHarvest, multi_eq_ignore_ascii_case};

const DELIMITER_LIST: [char; 6] = ['=', '>', '<', '(', ')', '\"'];
const COMPARISON_LIST: [&str; 3] = [">", "=", "<"];

/// A node type that can take a query and read it, advancing the current read state
/// and returning an instance of the node.
trait QueryProcessor {
    fn process(query: &mut VecDeque<String>, regex_options: &QueryOptions) -> QueryResult<Self>
    where
        Self: Sized;
}

/// Process a new regex given a `base` string and some settings.
///
/// TODO: Push this into a struct so I don't have to throw the options around so much.
fn new_regex(base: &str, regex_options: &QueryOptions) -> QueryResult<Regex> {
    let QueryOptions {
        whole_word: is_searching_whole_word,
        ignore_case: is_ignoring_case,
        use_regex: is_searching_with_regex,
    } = regex_options;
    let escaped_regex: String; // Needed for ownership reasons.

    let final_regex_string = &format!(
        "{}{}{}{}",
        if *is_searching_whole_word { "^" } else { "" },
        if *is_ignoring_case { "(?i)" } else { "" },
        if !(*is_searching_with_regex) {
            escaped_regex = regex::escape(base);
            &escaped_regex
        } else {
            base
        },
        if *is_searching_whole_word { "$" } else { "" },
    );

    Ok(Regex::new(final_regex_string)?)
}

/// Options when creating a new query.
#[derive(PartialEq, Eq)]
pub struct QueryOptions {
    /// Whether we only allow matches on the entire word.
    pub whole_word: bool,

    /// Whether to ignore case-sensitivity when searching. On by default.
    pub ignore_case: bool,

    /// Whether we should use regex syntax when searching. If not set, then it
    /// should treat everything as a literal string.
    pub use_regex: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            ignore_case: true,
            whole_word: false,
            use_regex: false,
        }
    }
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
pub(crate) fn parse_query(search_query: &str, options: &QueryOptions) -> QueryResult<ProcessQuery> {
    fn process_string_to_filter(
        query: &mut VecDeque<String>, options: &QueryOptions,
    ) -> QueryResult<ProcessQuery> {
        let lhs = Or::process(query, options)?;
        let mut list_of_ors = vec![lhs];

        while query.front().is_some() {
            list_of_ors.push(Or::process(query, options)?);
        }

        Ok(ProcessQuery { query: list_of_ors })
    }

    let mut split_query = VecDeque::new();

    search_query.split_whitespace().for_each(|s| {
        // From https://stackoverflow.com/a/56923739 get a split but include the parentheses
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

    process_string_to_filter(&mut split_query, options)
}

#[derive(Debug)]
pub struct ProcessQuery {
    /// Remember, AND > OR, but AND must come after OR when we parse.
    query: Vec<Or>,
}

impl ProcessQuery {
    pub(crate) fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        self.query
            .iter()
            .all(|ok| ok.check(process, is_using_command))
    }
}

#[derive(Debug)]
enum PrefixType {
    Pid,
    CpuPercentage,
    MemBytes,
    MemPercentage,
    ReadPerSecond,
    WritePerSecond,
    TotalRead,
    TotalWrite,
    Name,
    State,
    User,
    Time,
    #[cfg(unix)]
    Nice,
    Priority,
    #[cfg(feature = "gpu")]
    GpuPercentage,
    #[cfg(feature = "gpu")]
    GpuMemoryBytes,
    #[cfg(feature = "gpu")]
    GpuMemoryPercentage,
}

impl std::str::FromStr for PrefixType {
    type Err = QueryError;

    fn from_str(s: &str) -> QueryResult<Self> {
        use PrefixType::*;

        // TODO: Didn't add mem_bytes, total_read, and total_write
        // for now as it causes help to be clogged.

        let mut result = Name;
        if multi_eq_ignore_ascii_case!(s, "cpu" | "cpu%") {
            result = CpuPercentage;
        } else if multi_eq_ignore_ascii_case!(s, "mem" | "mem%") {
            result = MemPercentage;
        } else if multi_eq_ignore_ascii_case!(s, "memb") {
            result = MemBytes;
        } else if multi_eq_ignore_ascii_case!(s, "read" | "r/s" | "rps") {
            result = ReadPerSecond;
        } else if multi_eq_ignore_ascii_case!(s, "write" | "w/s" | "wps") {
            result = WritePerSecond;
        } else if multi_eq_ignore_ascii_case!(s, "tread" | "t.read") {
            result = TotalRead;
        } else if multi_eq_ignore_ascii_case!(s, "twrite" | "t.write") {
            result = TotalWrite;
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
                result = GpuMemoryBytes;
            } else if multi_eq_ignore_ascii_case!(s, "gmem%") {
                result = GpuMemoryPercentage;
            } else if multi_eq_ignore_ascii_case!(s, "gpu%") {
                result = GpuPercentage;
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
struct NumericalQuery {
    condition: QueryComparison,
    value: f64,
}

impl NumericalQuery {
    /// Compare `lhs` to the value in the query as `rhs`.
    fn check<I: Into<f64>>(&self, lhs: I) -> bool {
        let lhs: f64 = lhs.into();
        let rhs: f64 = self.value;

        match self.condition {
            QueryComparison::Equal => (lhs - rhs).abs() < f64::EPSILON,
            QueryComparison::Less => lhs < rhs,
            QueryComparison::Greater => lhs > rhs,
            QueryComparison::LessOrEqual => lhs <= rhs,
            QueryComparison::GreaterOrEqual => lhs >= rhs,
        }
    }
}

#[derive(Debug)]
struct TimeQuery {
    condition: QueryComparison,
    duration: Duration,
}

impl TimeQuery {
    /// Compare `lhs` to the value in the query as `rhs`.
    fn check(&self, lhs: Duration) -> bool {
        let rhs = self.duration;

        match self.condition {
            QueryComparison::Equal => lhs == rhs,
            QueryComparison::Less => lhs < rhs,
            QueryComparison::Greater => lhs > rhs,
            QueryComparison::LessOrEqual => lhs <= rhs,
            QueryComparison::GreaterOrEqual => lhs >= rhs,
        }
    }
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

    fn parse_query_no_options(query: &str) -> QueryResult<ProcessQuery> {
        parse_query(
            query,
            &QueryOptions {
                whole_word: false,
                ignore_case: false,
                use_regex: false,
            },
        )
    }

    #[test]
    fn basic_query() {
        let query = parse_query_no_options("test").unwrap();

        let exact_match = simple_process("test");
        let contains = simple_process("test string");
        let invalid = simple_process("no");

        assert!(query.check(&exact_match, false));
        assert!(query.check(&contains, false));
        assert!(!query.check(&invalid, false));
    }

    #[test]
    fn basic_or_query() {
        let query = parse_query_no_options("a or b").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let invalid = simple_process("c");

        assert!(query.check(&a, false));
        assert!(query.check(&b, false));
        assert!(!query.check(&invalid, false));
    }

    #[test]
    fn basic_and_query() {
        let query = parse_query_no_options("a and b").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let c = simple_process("c");
        let a_and_b = simple_process("a and b");

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&c, false));
        assert!(query.check(&a_and_b, false));
    }

    #[test]
    fn implied_and_query() {
        let query = parse_query_no_options("a b c").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let c = simple_process("c");
        let all = simple_process("a b c");

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&c, false));
        assert!(query.check(&all, false));
    }

    /// Ensure that quoted keywords are treated as strings. In this case, rather than `"a" OR "b"`, it should be treated
    /// as the string `"a or b"`.
    #[test]
    fn quoted_query() {
        let query = parse_query_no_options("a \"or\" b").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let or = simple_process("or");
        let valid = simple_process("a \"or\" b"); // This is valid as the query is "match a word with a, or, and b".
        let valid_2 = simple_process("a or b");
        let valid_3 = simple_process("a \"or\" b \"or\" c");

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&or, false));
        assert!(query.check(&valid, false));
        assert!(query.check(&valid_2, false));
        assert!(query.check(&valid_3, false));
    }

    /// Ensure that multi-word quoted keywords are treated as strings. In this case, rather than `"a" OR "b"`, it should be treated
    /// as the string `"a or b"`.
    #[test]
    fn quoted_multi_word_query() {
        let query = parse_query_no_options("\"a or b\"").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let or = simple_process("or");
        let valid = simple_process("a or b");
        let valid_2 = simple_process("a or b \"or\" c");
        let invalid_no_regex = simple_process("a \"or\" b"); // Invalid now as the query is one big string!

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&or, false));
        assert!(query.check(&valid, false));
        assert!(query.check(&valid_2, false));
        assert!(!query.check(&invalid_no_regex, false));
    }

    #[test]
    fn basic_cpu_query() {
        let query = parse_query_no_options("cpu > 50").unwrap();

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
        let query = parse_query_no_options("memb > 1 GiB").unwrap();

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
        let query = parse_query_no_options("(a or b) and (c or a)").unwrap();

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
        let query = parse_query_no_options("(cpu > 10 or cpu < 5) and (c or a)").unwrap();

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
        let query =
            parse_query_no_options("((cpu > 10 or cpu < 5) or d) and ((c or a) or d)").unwrap();

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

    #[test]
    fn ambiguous_precedence_1() {
        let query = parse_query_no_options("a and b or c").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let c = simple_process("c");

        assert!(!query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(query.check(&c, false));
    }

    #[test]
    fn ambiguous_precedence_2() {
        let query = parse_query_no_options("a or b and c").unwrap();

        let a = simple_process("a");
        let b = simple_process("b");
        let c = simple_process("c");

        assert!(query.check(&a, false));
        assert!(!query.check(&b, false));
        assert!(!query.check(&c, false));
    }

    /// Test if a complicated query even parses.
    #[test]
    fn parse_complicated_query() {
        parse_query_no_options(
            "cpu > 10.5 AND (memb = 1 MiB || state = sleeping) and (a or b) and (read >= 0 or write >= 0)",

        )
        .unwrap();
    }

    /// Test empty quotes works.
    #[test]
    fn parse_empty_quotes() {
        parse_query_no_options("\"\"").unwrap();
    }

    /// Test unfinished quotes error.
    #[test]
    fn parse_unfinished_quotes() {
        parse_query_no_options("\"").unwrap_err();
    }

    /// Test a fix for a bug with closing quotations. The problem seems to arise from quotes being used as an argument
    /// to a prefix... but this should probably be valid.
    #[test]
    fn parse_nested_closing_quotes() {
        parse_query_no_options("state = \"test\"").unwrap();
        parse_query_no_options("state = \"2 words\"").unwrap();
        parse_query_no_options("(memb = 1 MiB || state = \"test\")").unwrap();
        parse_query_no_options("(memb = 1 MiB || state = \"2 words\")").unwrap();
    }

    // TODO: Add this after fixed.
    // /// Test if units can ignore spaces from their preceding value.
    // #[test]
    // fn units_with_and_without_spaces() {}

    #[test]
    fn invalid_query_1() {
        parse_query_no_options("state =").unwrap_err();
        parse_query_no_options("a or").unwrap_err();
        parse_query_no_options("a >").unwrap_err();
    }

    // /// Test keywords.
    // ///
    // /// TODO: Should these be invalid...?
    // #[test]
    // fn invalid_query_2() {
    //     parse_query_no_options("or").unwrap_err();
    //     parse_query_no_options("and").unwrap_err();
    //     parse_query_no_options("a or >").unwrap_err();
    //     parse_query_no_options("a and >").unwrap_err();
    // }
}
