use crate::{
    data_conversion::ConvertedProcessData,
    utils::error::{BottomError, Result},
};

#[derive(Debug)]
pub struct Query {
    pub query: And,
}

impl Query {
    pub fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> Result<()> {
        self.query.process_regexes(
            is_searching_whole_word,
            is_ignoring_case,
            is_searching_with_regex,
        )
    }

    pub fn check(&self, process: &ConvertedProcessData) -> bool {
        self.query.check(process)
    }
}

#[derive(Debug)]
pub struct And {
    pub lhs: Or,
    pub rhs: Option<Box<Or>>,
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

    pub fn check(&self, process: &ConvertedProcessData) -> bool {
        if let Some(rhs) = &self.rhs {
            self.lhs.check(process) && rhs.check(process)
        } else {
            self.lhs.check(process)
        }
    }
}

#[derive(Debug)]
pub struct Or {
    pub lhs: Prefix,
    pub rhs: Option<Box<Prefix>>,
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

    pub fn check(&self, process: &ConvertedProcessData) -> bool {
        if let Some(rhs) = &self.rhs {
            self.lhs.check(process) || rhs.check(process)
        } else {
            self.lhs.check(process)
        }
    }
}

#[derive(Debug)]
pub enum PrefixType {
    Pid,
    Cpu,
    Mem,
    Rps,
    Wps,
    TRead,
    TWrite,
    Name,
    __Nonexhaustive,
}

impl std::str::FromStr for PrefixType {
    type Err = BottomError;

    fn from_str(s: &str) -> Result<Self> {
        use PrefixType::*;

        let lower_case = s.to_lowercase();
        match lower_case.as_str() {
            "cpu" => Ok(Cpu),
            "mem" => Ok(Mem),
            "r" => Ok(Rps),
            "w" => Ok(Wps),
            "read" => Ok(TRead),
            "write" => Ok(TWrite),
            "pid" => Ok(Pid),
            _ => Ok(Name),
        }
    }
}

#[derive(Debug)]
pub struct Prefix {
    pub and: Option<Box<And>>,
    pub regex_prefix: Option<(PrefixType, StringQuery)>,
    pub compare_prefix: Option<(PrefixType, NumericalQuery)>,
}

impl Prefix {
    pub fn process_regexes(
        &mut self, is_searching_whole_word: bool, is_ignoring_case: bool,
        is_searching_with_regex: bool,
    ) -> Result<()> {
        if let Some(and) = &mut self.and {
            return and.process_regexes(
                is_searching_whole_word,
                is_ignoring_case,
                is_searching_with_regex,
            );
        } else if let Some((prefix_type, query_content)) = &mut self.regex_prefix {
            if let StringQuery::Value(regex_string) = query_content {
                match prefix_type {
                    PrefixType::Pid | PrefixType::Name => {
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
        }

        Ok(())
    }

    pub fn check(&self, process: &ConvertedProcessData) -> bool {
        fn matches_condition(condition: &QueryComparison, lhs: f64, rhs: f64) -> bool {
            match condition {
                QueryComparison::Equal => lhs == rhs,
                QueryComparison::Less => lhs < rhs,
                QueryComparison::Greater => lhs > rhs,
                QueryComparison::LessOrEqual => lhs <= rhs,
                QueryComparison::GreaterOrEqual => lhs >= rhs,
            }
        }

        if let Some(and) = &self.and {
            and.check(process)
        } else if let Some((prefix_type, query_content)) = &self.regex_prefix {
            if let StringQuery::Regex(r) = query_content {
                match prefix_type {
                    PrefixType::Name => r.is_match(process.name.as_str()),
                    PrefixType::Pid => r.is_match(process.pid.to_string().as_str()),
                    _ => true,
                }
            } else {
                true
            }
        } else if let Some((prefix_type, numerical_query)) = &self.compare_prefix {
            match prefix_type {
                PrefixType::Cpu => matches_condition(
                    &numerical_query.condition,
                    process.cpu_usage,
                    numerical_query.value,
                ),
                PrefixType::Mem => matches_condition(
                    &numerical_query.condition,
                    process.mem_usage,
                    numerical_query.value,
                ),
                PrefixType::Rps => matches_condition(
                    &numerical_query.condition,
                    process.rps_f64,
                    numerical_query.value,
                ),
                PrefixType::Wps => matches_condition(
                    &numerical_query.condition,
                    process.wps_f64,
                    numerical_query.value,
                ),
                PrefixType::TRead => matches_condition(
                    &numerical_query.condition,
                    process.tr_f64,
                    numerical_query.value,
                ),
                PrefixType::TWrite => matches_condition(
                    &numerical_query.condition,
                    process.tw_f64,
                    numerical_query.value,
                ),
                _ => true,
            }
        } else {
            true
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
