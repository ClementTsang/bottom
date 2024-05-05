use tui::{
    style::Color,
    widgets::{
        canvas::{Line as CanvasLine, Points},
        GraphType,
    },
};

use super::{Context, Dataset, Point, TimeChart};
use crate::utils::general::partial_ordering;

impl TimeChart<'_> {
    pub(crate) fn draw_points(&self, ctx: &mut Context<'_>) {
        // Idea is to:
        // - Go over all datasets, determine *where* a point will be drawn.
        // - Last point wins for what gets drawn.
        // - We set _all_ points for all datasets before actually rendering.
        //
        // By doing this, it's a bit more efficient from my experience than looping
        // over each dataset and rendering a new layer each time.
        //
        // See <https://github.com/ClementTsang/bottom/pull/918> and <https://github.com/ClementTsang/bottom/pull/937>
        // for the original motivation.
        //
        // We also additionally do some interpolation logic because we may get caught missing some points
        // when drawing, but we generally want to avoid jarring gaps between the edges when there's
        // a point that is off screen and so a line isn't drawn (right edge generally won't have this issue
        // issue but it can happen in some cases).

        for dataset in &self.datasets {
            let color = dataset.style.fg.unwrap_or(Color::Reset);

            let start_bound = self.x_axis.bounds[0];
            let end_bound = self.x_axis.bounds[1];

            let (start_index, interpolate_start) = get_start(dataset, start_bound);
            let (end_index, interpolate_end) = get_end(dataset, end_bound);

            let data_slice = &dataset.data[start_index..end_index];

            if let Some(interpolate_start) = interpolate_start {
                if let (Some(older_point), Some(newer_point)) = (
                    dataset.data.get(interpolate_start),
                    dataset.data.get(interpolate_start + 1),
                ) {
                    let interpolated_point = (
                        self.x_axis.bounds[0],
                        interpolate_point(older_point, newer_point, self.x_axis.bounds[0]),
                    );

                    if let GraphType::Line = dataset.graph_type {
                        ctx.draw(&CanvasLine {
                            x1: interpolated_point.0,
                            y1: interpolated_point.1,
                            x2: newer_point.0,
                            y2: newer_point.1,
                            color,
                        });
                    } else {
                        ctx.draw(&Points {
                            coords: &[interpolated_point],
                            color,
                        });
                    }
                }
            }

            if let GraphType::Line = dataset.graph_type {
                for data in data_slice.windows(2) {
                    ctx.draw(&CanvasLine {
                        x1: data[0].0,
                        y1: data[0].1,
                        x2: data[1].0,
                        y2: data[1].1,
                        color,
                    });
                }
            } else {
                ctx.draw(&Points {
                    coords: data_slice,
                    color,
                });
            }

            if let Some(interpolate_end) = interpolate_end {
                if let (Some(older_point), Some(newer_point)) = (
                    dataset.data.get(interpolate_end - 1),
                    dataset.data.get(interpolate_end),
                ) {
                    let interpolated_point = (
                        self.x_axis.bounds[1],
                        interpolate_point(older_point, newer_point, self.x_axis.bounds[1]),
                    );

                    if let GraphType::Line = dataset.graph_type {
                        ctx.draw(&CanvasLine {
                            x1: older_point.0,
                            y1: older_point.1,
                            x2: interpolated_point.0,
                            y2: interpolated_point.1,
                            color,
                        });
                    } else {
                        ctx.draw(&Points {
                            coords: &[interpolated_point],
                            color,
                        });
                    }
                }
            }
        }
    }
}

/// Returns the start index and potential interpolation index given the start time and the dataset.
fn get_start(dataset: &Dataset<'_>, start_bound: f64) -> (usize, Option<usize>) {
    match dataset
        .data
        .binary_search_by(|(x, _y)| partial_ordering(x, &start_bound))
    {
        Ok(index) => (index, None),
        Err(index) => (index, index.checked_sub(1)),
    }
}

/// Returns the end position and potential interpolation index given the end time and the dataset.
fn get_end(dataset: &Dataset<'_>, end_bound: f64) -> (usize, Option<usize>) {
    match dataset
        .data
        .binary_search_by(|(x, _y)| partial_ordering(x, &end_bound))
    {
        // In the success case, this means we found an index. Add one since we want to include this index and we
        // expect to use the returned index as part of a (m..n) range.
        Ok(index) => (index.saturating_add(1), None),
        // In the fail case, this means we did not find an index, and the returned index is where one would *insert*
        // the location. This index is where one would insert to fit inside the dataset - and since this is an end
        // bound, index is, in a sense, already "+1" for our range later.
        Err(index) => (index, {
            let sum = index.checked_add(1);
            match sum {
                Some(s) if s < dataset.data.len() => sum,
                _ => None,
            }
        }),
    }
}

/// Returns the y-axis value for a given `x`, given two points to draw a line between.
fn interpolate_point(older_point: &Point, newer_point: &Point, x: f64) -> f64 {
    let delta_x = newer_point.0 - older_point.0;
    let delta_y = newer_point.1 - older_point.1;
    let slope = delta_y / delta_x;

    (older_point.1 + (x - older_point.0) * slope).max(0.0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_chart_test_interpolation() {
        let data = [(-3.0, 8.0), (-1.0, 6.0), (0.0, 5.0)];

        assert_eq!(interpolate_point(&data[1], &data[2], 0.0), 5.0);
        assert_eq!(interpolate_point(&data[1], &data[2], -0.25), 5.25);
        assert_eq!(interpolate_point(&data[1], &data[2], -0.5), 5.5);
        assert_eq!(interpolate_point(&data[0], &data[1], -1.0), 6.0);
        assert_eq!(interpolate_point(&data[0], &data[1], -1.5), 6.5);
        assert_eq!(interpolate_point(&data[0], &data[1], -2.0), 7.0);
        assert_eq!(interpolate_point(&data[0], &data[1], -2.5), 7.5);
        assert_eq!(interpolate_point(&data[0], &data[1], -3.0), 8.0);
    }

    #[test]
    fn time_chart_empty_dataset() {
        let data = [];
        let dataset = Dataset::default().data(&data);

        assert_eq!(get_start(&dataset, -100.0), (0, None));
        assert_eq!(get_start(&dataset, -3.0), (0, None));

        assert_eq!(get_end(&dataset, 0.0), (0, None));
        assert_eq!(get_end(&dataset, 100.0), (0, None));
    }

    #[test]
    fn time_chart_test_data_trimming() {
        let data = [
            (-3.0, 8.0),
            (-2.5, 15.0),
            (-2.0, 9.0),
            (-1.0, 6.0),
            (0.0, 5.0),
        ];
        let dataset = Dataset::default().data(&data);

        // Test start point cases (miss and hit)
        assert_eq!(get_start(&dataset, -100.0), (0, None));
        assert_eq!(get_start(&dataset, -3.0), (0, None));
        assert_eq!(get_start(&dataset, -2.8), (1, Some(0)));
        assert_eq!(get_start(&dataset, -2.5), (1, None));
        assert_eq!(get_start(&dataset, -2.4), (2, Some(1)));

        // Test end point cases (miss and hit)
        assert_eq!(get_end(&dataset, -2.5), (2, None));
        assert_eq!(get_end(&dataset, -2.4), (2, Some(3)));
        assert_eq!(get_end(&dataset, -1.4), (3, Some(4)));
        assert_eq!(get_end(&dataset, -1.0), (4, None));
        assert_eq!(get_end(&dataset, 0.0), (5, None));
        assert_eq!(get_end(&dataset, 1.0), (5, None));
        assert_eq!(get_end(&dataset, 100.0), (5, None));
    }
}
