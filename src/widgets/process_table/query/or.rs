use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter},
};

use crate::{
    collection::processes::ProcessHarvest,
    widgets::query::{
        And, COMPARISON_LIST, Prefix, QueryProcessor, QueryResult, error::QueryError,
    },
};

/// A node where either the left hand side or the right hand side are considered.
/// Note that the right hand side is optional, as that's how I implemented it a long time ago.
#[derive(Default)]
pub(super) struct Or {
    pub(super) lhs: And,
    pub(super) rhs: Option<Box<And>>,
}

impl Or {
    pub(super) fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> QueryResult<()> {
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

    pub(super) fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        if let Some(rhs) = &self.rhs {
            self.lhs.check(process, is_using_command) || rhs.check(process, is_using_command)
        } else {
            self.lhs.check(process, is_using_command)
        }
    }
}

impl Debug for Or {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.rhs {
            Some(rhs) => f.write_fmt(format_args!("({:?} OR {:?})", self.lhs, rhs)),
            None => f.write_fmt(format_args!("{:?}", self.lhs)),
        }
    }
}

impl QueryProcessor for Or {
    fn process(query: &mut VecDeque<String>) -> QueryResult<Self>
    where
        Self: Sized,
    {
        const OR_LIST: [&str; 2] = ["or", "||"];

        let mut lhs = And::process(query)?;
        let mut rhs: Option<Box<And>> = None;

        while let Some(queue_top) = query.front() {
            let current_lowercase = queue_top.to_lowercase();
            if OR_LIST.contains(&current_lowercase.as_str()) {
                query.pop_front();
                rhs = Some(Box::new(And::process(query)?));

                if let Some(queue_next) = query.front() {
                    if OR_LIST.contains(&queue_next.to_lowercase().as_str()) {
                        // Must merge LHS and RHS
                        lhs = And {
                            lhs: Prefix {
                                or: Some(Box::new(Or { lhs, rhs })),
                                regex_prefix: None,
                                compare_prefix: None,
                                negate: false,
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
}
