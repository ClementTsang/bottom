/// Filters used by widgets to filter out certain entries.
/// TODO: Move this out maybe?
#[derive(Debug, Clone)]
pub struct Filter {
    /// Whether the filter _accepts_ all entries that match `list`,
    /// or _denies_ any entries that match it.
    pub is_list_ignored: bool, // TODO: Maybe change to "ignore_matches"?

    /// The list of regexes to match against. Whether it goes through
    /// the filter or not depends on `is_list_ignored`.
    pub list: Vec<regex::Regex>,
}

impl Filter {
    /// Whether the filter should keep the entry or reject it.
    #[inline]
    pub(crate) fn keep_entry(&self, value: &str) -> bool {
        if self.has_match(value) {
            // If a match is found, then if we wanted to ignore if we match, return false.
            // If we want to keep if we match, return true. Thus, return the
            // inverse of `is_list_ignored`.
            !self.is_list_ignored
        } else {
            self.is_list_ignored
        }
    }

    /// Whether there is a filter that matches the result.
    #[inline]
    pub(crate) fn has_match(&self, value: &str) -> bool {
        self.list.iter().any(|regex| regex.is_match(value))
    }
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use super::*;

    /// Test based on the issue in <https://github.com/ClementTsang/bottom/pull/1037>.
    #[test]
    fn filter_is_list_ignored() {
        let results = [
            "CPU socket temperature",
            "wifi_0",
            "motherboard temperature",
            "amd gpu",
        ];

        let ignore_true = Filter {
            is_list_ignored: true,
            list: vec![Regex::new("temperature").unwrap()],
        };

        assert_eq!(
            results
                .into_iter()
                .filter(|r| ignore_true.keep_entry(r))
                .collect::<Vec<_>>(),
            vec!["wifi_0", "amd gpu"]
        );

        let ignore_false = Filter {
            is_list_ignored: false,
            list: vec![Regex::new("temperature").unwrap()],
        };

        assert_eq!(
            results
                .into_iter()
                .filter(|r| ignore_false.keep_entry(r))
                .collect::<Vec<_>>(),
            vec!["CPU socket temperature", "motherboard temperature"]
        );

        let multi_true = Filter {
            is_list_ignored: true,
            list: vec![
                Regex::new("socket").unwrap(),
                Regex::new("temperature").unwrap(),
            ],
        };

        assert_eq!(
            results
                .into_iter()
                .filter(|r| multi_true.keep_entry(r))
                .collect::<Vec<_>>(),
            vec!["wifi_0", "amd gpu"]
        );

        let multi_false = Filter {
            is_list_ignored: false,
            list: vec![
                Regex::new("socket").unwrap(),
                Regex::new("temperature").unwrap(),
            ],
        };

        assert_eq!(
            results
                .into_iter()
                .filter(|r| multi_false.keep_entry(r))
                .collect::<Vec<_>>(),
            vec!["CPU socket temperature", "motherboard temperature"]
        );
    }
}
