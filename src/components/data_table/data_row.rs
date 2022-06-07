use std::{cmp::max, fmt::Display};

use tui::widgets::Row;

pub trait ToDataRow {
    fn to_data_row<'a>(&'a self) -> Row<'a>;

    fn col_widths(&self) -> Vec<usize>; // Returning an iter would be preferable, but its fine for now.
}

impl<T: Display> ToDataRow for Vec<T> {
    fn to_data_row<'a>(&'a self) -> Row<'a> {
        Row::new(self.iter().map(|c| c.to_string()))
    }

    fn col_widths(&self) -> Vec<usize> {
        self.iter().map(|c| c.to_string().len()).collect()
    }
}

pub trait MaxColWidth {
    fn max_col_widths(&self) -> Vec<usize>;
}

impl<T: ToDataRow> MaxColWidth for [T] {
    fn max_col_widths(&self) -> Vec<usize> {
        if let Some(first) = self.first() {
            let mut best = first.col_widths();
            for row in self.iter().skip(1) {
                let candidate = row.col_widths();
                for (b, c) in best.iter_mut().zip(candidate) {
                    *b = max(*b, c);
                }
            }

            best
        } else {
            vec![]
        }
    }
}
