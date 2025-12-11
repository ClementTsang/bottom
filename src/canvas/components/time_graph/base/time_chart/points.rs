use itertools::Itertools;
use tui::{
    style::Color,
    widgets::{
        GraphType,
        canvas::{Line as CanvasLine, Points},
    },
};

use super::{Context, Data, Point, TimeChart, canvas::FilledLine};

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
        // See https://github.com/ClementTsang/bottom/pull/918 and
        // https://github.com/ClementTsang/bottom/pull/937 for the original motivation.
        //
        // We also additionally do some interpolation logic because we may get caught
        // missing some points when drawing, but we generally want to avoid
        // jarring gaps between the edges when there's a point that is off
        // screen and so a line isn't drawn (right edge generally won't have this issue
        // issue but it can happen in some cases).

        for dataset in &self.datasets {
            match &dataset.data {
                Data::Some { times, values } => {
                    self.draw_dataset_iter(ctx, dataset, times, values.iter_along_base(times));
                }
                Data::Custom { times, values } => {
                    // Zip time and values
                    self.draw_dataset_iter(ctx, dataset, times, times.iter().zip(values.iter()));
                }
                Data::None => continue,
            }
        }
    }

    fn draw_dataset_iter<'a, I>(
        &self, ctx: &mut Context<'_>, dataset: &super::Dataset<'_>, times: &[std::time::Instant],
        iterator: I,
    ) where
        I: Iterator<Item = (&'a std::time::Instant, &'a f64)> + DoubleEndedIterator,
    {
        let Some(current_time) = times.last() else {
            return;
        };

        let color = dataset.style.fg.unwrap_or(Color::Reset);
        let left_edge = self.x_axis.bounds.get_bounds()[0];

        // TODO: (points_rework_v1) Can we instead modify the range so it's based on the epoch rather than having to convert?
        // TODO: (points_rework_v1) Is this efficient? Or should I prune using take_while first?
        for (curr, next) in iterator
            .rev()
            .map(|(&time, &val)| {
                let from_start = -(current_time.duration_since(time).as_millis() as f64);
                let val = if dataset.inverted { -val } else { val };

                // XXX: Should this be generic over dataset.graph_type instead? That would allow us to move
                // transformations behind a type - however, that also means that there's some complexity added.
                (from_start, self.scaling.scale(val))
            })
            .tuple_windows()
        {
            if curr.0 == left_edge {
                // The current point hits the left edge. Draw just the current point and halt.
                ctx.draw(&Points {
                    coords: &[curr],
                    color,
                });

                break;
            } else if next.0 < left_edge {
                // The next point goes past the left edge. Interpolate a point + the line and halt.
                let interpolated = interpolate_point(&next, &curr, left_edge);

                if dataset.filled {
                    ctx.draw(&FilledLine {
                        x1: curr.0,
                        y1: curr.1,
                        x2: left_edge,
                        y2: interpolated,
                        color,
                        baseline: Some(0.0),
                    });
                } else {
                    ctx.draw(&CanvasLine {
                        x1: curr.0,
                        y1: curr.1,
                        x2: left_edge,
                        y2: interpolated,
                        color,
                    });
                }

                break;
            } else {
                // Draw the current point and the line to the next point.
                if let GraphType::Line = dataset.graph_type {
                    if dataset.filled {
                        ctx.draw(&FilledLine {
                            x1: curr.0,
                            y1: curr.1,
                            x2: next.0,
                            y2: next.1,
                            color,
                            baseline: Some(0.0),
                        });
                    } else {
                        ctx.draw(&CanvasLine {
                            x1: curr.0,
                            y1: curr.1,
                            x2: next.0,
                            y2: next.1,
                            color,
                        });
                    }
                } else {
                    ctx.draw(&Points {
                        coords: &[curr],
                        color,
                    });
                }
            }
        }
    }
}

/// Returns the y-axis value for a given `x`, given two points to draw a line
/// between.
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
}
