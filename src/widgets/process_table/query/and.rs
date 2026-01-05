use std::collections::VecDeque;

use crate::{
    collection::processes::ProcessHarvest,
    widgets::query::{
        COMPARISON_LIST, Or, Prefix, QueryOptions, QueryProcessor, QueryResult, error::QueryError,
    },
};

/// A node where both the left hand side or the right hand side are considered.
/// Note that the right hand side is optional, as that's how I implemented it a long time ago.
#[derive(Debug)]
pub(super) struct And {
    pub(super) lhs: Prefix,
    // TODO: Maybe don't need to box rhs?
    pub(super) rhs: Option<Box<Prefix>>,
}

impl And {
    pub(super) fn check(&self, process: &ProcessHarvest, is_using_command: bool) -> bool {
        if let Some(rhs) = &self.rhs {
            self.lhs.check(process, is_using_command) && rhs.check(process, is_using_command)
        } else {
            self.lhs.check(process, is_using_command)
        }
    }
}

impl QueryProcessor for And {
    fn process(query: &mut VecDeque<String>, options: &QueryOptions) -> QueryResult<Self>
    where
        Self: Sized,
    {
        const AND_LIST: [&str; 2] = ["and", "&&"];

        let mut lhs = Prefix::process(query, options)?;
        let mut rhs: Option<Box<Prefix>> = None;

        while let Some(queue_top) = query.front() {
            let current_lowercase = queue_top.to_lowercase();
            if AND_LIST.contains(&current_lowercase.as_str()) {
                query.pop_front();

                rhs = Some(Box::new(Prefix::process(query, options)?));

                if let Some(next_queue_top) = query.front() {
                    if AND_LIST.contains(&next_queue_top.to_lowercase().as_str()) {
                        // Must merge LHS and RHS
                        lhs = Prefix::Or(Box::new(Or {
                            lhs: And { lhs, rhs },
                            rhs: None,
                        }));
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
}
