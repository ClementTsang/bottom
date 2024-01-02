use tui::layout::{Direction, Rect};

use crate::canvas::LayoutConstraint;

pub(super) fn get_constraints(
    direction: Direction, constraints: &[LayoutConstraint], area: Rect,
) -> Vec<Rect> {
    // Order of operations:
    // - Ratios first + canvas-handled (which is just zero)
    // - Then any flex-grows to take up remaining space; divide amongst remaining
    //   hand out any remaining space

    #[derive(Debug, Default, Clone, Copy)]
    struct Size {
        width: u16,
        height: u16,
    }

    impl Size {
        fn shrink_width(&mut self, amount: u16) {
            self.width -= amount;
        }

        fn shrink_height(&mut self, amount: u16) {
            self.height -= amount;
        }
    }

    let mut bounds = Size {
        width: area.width,
        height: area.height,
    };
    let mut sizes = vec![Size::default(); constraints.len()];
    let mut grow = vec![];
    let mut num_non_ch = 0;

    for (itx, (constraint, size)) in constraints.iter().zip(sizes.iter_mut()).enumerate() {
        match constraint {
            LayoutConstraint::Ratio(a, b) => {
                match direction {
                    Direction::Horizontal => {
                        let amount = (((area.width as u32) * (*a)) / (*b)) as u16;
                        bounds.shrink_width(amount);
                        size.width = amount;
                        size.height = area.height;
                    }
                    Direction::Vertical => {
                        let amount = (((area.height as u32) * (*a)) / (*b)) as u16;
                        bounds.shrink_height(amount);
                        size.width = area.width;
                        size.height = amount;
                    }
                }
                num_non_ch += 1;
            }
            LayoutConstraint::Grow => {
                // Mark it as grow in the vector and handle in second pass.
                grow.push(itx);
                num_non_ch += 1;
            }
            LayoutConstraint::CanvasHandled => {
                // Do nothing in this case. It's already 0.
            }
        }
    }

    if !grow.is_empty() {
        match direction {
            Direction::Horizontal => {
                let width = bounds.width / grow.len() as u16;
                bounds.shrink_width(width * grow.len() as u16);
                for g in grow {
                    sizes[g] = Size {
                        width,
                        height: area.height,
                    };
                }
            }
            Direction::Vertical => {
                let height = bounds.height / grow.len() as u16;
                bounds.shrink_height(height * grow.len() as u16);
                for g in grow {
                    sizes[g] = Size {
                        width: area.width,
                        height,
                    };
                }
            }
        }
    }

    if num_non_ch > 0 {
        match direction {
            Direction::Horizontal => {
                let per_item = bounds.width / num_non_ch;
                let mut remaining_width = bounds.width % num_non_ch;
                for (size, constraint) in sizes.iter_mut().zip(constraints) {
                    match constraint {
                        LayoutConstraint::CanvasHandled => {}
                        LayoutConstraint::Grow | LayoutConstraint::Ratio(_, _) => {
                            if remaining_width > 0 {
                                size.width += per_item + 1;
                                remaining_width -= 1;
                            } else {
                                size.width += per_item;
                            }
                        }
                    }
                }
            }
            Direction::Vertical => {
                let per_item = bounds.height / num_non_ch;
                let mut remaining_height = bounds.height % num_non_ch;
                for (size, constraint) in sizes.iter_mut().zip(constraints) {
                    match constraint {
                        LayoutConstraint::CanvasHandled => {}
                        LayoutConstraint::Grow | LayoutConstraint::Ratio(_, _) => {
                            if remaining_height > 0 {
                                size.height += per_item + 1;
                                remaining_height -= 1;
                            } else {
                                size.height += per_item;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut curr_x = area.x;
    let mut curr_y = area.y;
    sizes
        .into_iter()
        .map(|size| {
            let rect = Rect::new(curr_x, curr_y, size.width, size.height);
            match direction {
                Direction::Horizontal => {
                    curr_x += size.width;
                }
                Direction::Vertical => {
                    curr_y += size.height;
                }
            }

            rect
        })
        .collect()
}

#[cfg(test)]
mod test {
    // TODO: Add some tests.
}
