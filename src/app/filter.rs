#[derive(Debug, Clone)]
pub struct Filter {
    // TODO: Maybe change to "ignore_matches"?
    pub is_list_ignored: bool,
    pub list: Vec<regex::Regex>,
}

impl Filter {
    /// Whether the filter should keep the entry or reject it.
    #[inline]
    pub(crate) fn keep_entry(&self, value: &str) -> bool {
        self.list
            .iter()
            .find(|regex| regex.is_match(value))
            .map(|_| !self.is_list_ignored)
            .unwrap_or(self.is_list_ignored)
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
    }
}
