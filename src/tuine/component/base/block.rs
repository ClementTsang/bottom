use std::{borrow::Cow, marker::PhantomData};

use tui::{backend::Backend, layout::Rect, style::Style, widgets::Borders, Frame};

use crate::tuine::{
    Bounds, DrawContext, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

/// A set of styles for a [`Block`].
#[derive(Clone, Debug, Default)]
pub struct StyleSheet {
    pub border: Style,
}

struct BorderOffsets {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

/// A [`Block`] is a widget that draws a border around a child [`Component`], as well as optional
/// titles.
pub struct Block<Message, Child>
where
    Child: TmpComponent<Message>,
{
    _pd: PhantomData<Message>,
    child: Option<Child>,
    borders: Borders,
    style_sheet: StyleSheet,
    left_text: Option<Cow<'static, str>>,
    right_text: Option<Cow<'static, str>>,
}

impl<Message, Child> Block<Message, Child>
where
    Child: TmpComponent<Message>,
{
    pub fn with_child(child: Child) -> Self {
        Self {
            _pd: Default::default(),
            child: Some(child),
            borders: Borders::all(),
            style_sheet: Default::default(),
            left_text: None,
            right_text: None,
        }
    }

    pub fn child(mut self, child: Option<Child>) -> Self {
        self.child = child;
        self
    }

    pub fn style(mut self, style: StyleSheet) -> Self {
        self.style_sheet = style;
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    fn border_offsets(&self) -> BorderOffsets {
        fn border_val(has_val: bool) -> u16 {
            if has_val {
                1
            } else {
                0
            }
        }

        BorderOffsets {
            left: border_val(self.borders.intersects(Borders::LEFT)),
            right: border_val(self.borders.intersects(Borders::RIGHT)),
            top: border_val(
                self.borders.intersects(Borders::TOP)
                    || self.left_text.is_some()
                    || self.right_text.is_some(),
            ),
            bottom: border_val(self.borders.intersects(Borders::BOTTOM)),
        }
    }
}

impl<Message, Child> TmpComponent<Message> for Block<Message, Child>
where
    Child: TmpComponent<Message>,
{
    fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        let rect = draw_ctx.global_rect();
        if rect.area() > 0 {
            frame.render_widget(
                tui::widgets::Block::default()
                    .borders(self.borders)
                    .border_style(self.style_sheet.border),
                rect,
            );
            if let Some(child) = &mut self.child {
                if let Some(child_draw_ctx) = draw_ctx.children().next() {
                    child.draw(state_ctx, &child_draw_ctx, frame)
                }
            }
        }
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        if let Some(child_draw_ctx) = draw_ctx.children().next() {
            if let Some(child) = &mut self.child {
                return child.on_event(state_ctx, &child_draw_ctx, event, messages);
            }
        }

        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        if let Some(child) = &self.child {
            let BorderOffsets {
                left: left_offset,
                right: right_offset,
                top: top_offset,
                bottom: bottom_offset,
            } = self.border_offsets();

            let vertical_offset = top_offset + bottom_offset;
            let horizontal_offset = left_offset + right_offset;

            if bounds.max_height > vertical_offset && bounds.max_width > horizontal_offset {
                let max_width = bounds.max_width - horizontal_offset;
                let max_height = bounds.max_height - vertical_offset;

                let child_bounds = Bounds {
                    min_width: bounds.min_width,
                    min_height: bounds.min_height,
                    max_width,
                    max_height,
                };
                let mut child_node = LayoutNode::default();
                let child_size = child.layout(child_bounds, &mut child_node);

                child_node.rect =
                    Rect::new(left_offset, top_offset, child_size.width, child_size.height);
                node.children = vec![child_node];

                Size {
                    width: child_size.width + horizontal_offset,
                    height: child_size.height + vertical_offset,
                }
            } else {
                Size {
                    width: 0,
                    height: 0,
                }
            }
        } else {
            Size {
                width: 0,
                height: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tuine::Empty;

    use super::*;

    fn assert_border_offset(block: Block<(), Empty>, left: u16, right: u16, top: u16, bottom: u16) {
        let offsets = block.border_offsets();
        assert_eq!(offsets.left, left, "left offset should be equal");
        assert_eq!(offsets.right, right, "right offset should be equal");
        assert_eq!(offsets.top, top, "top offset should be equal");
        assert_eq!(offsets.bottom, bottom, "bottom offset should be equal");
    }

    #[test]
    fn empty_border_offset() {
        let block: Block<(), Empty> = Block::with_child(Empty::default()).borders(Borders::empty());
        assert_border_offset(block, 0, 0, 0, 0);
    }

    #[test]
    fn all_border_offset() {
        let block: Block<(), Empty> = Block::with_child(Empty::default());
        assert_border_offset(block, 1, 1, 1, 1);
    }

    #[test]
    fn horizontal_border_offset() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::LEFT.union(Borders::RIGHT));
        assert_border_offset(block, 1, 1, 0, 0);
    }

    #[test]
    fn vertical_border_offset() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::BOTTOM.union(Borders::TOP));
        assert_border_offset(block, 0, 0, 1, 1);
    }

    #[test]
    fn top_right() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::RIGHT.union(Borders::TOP));
        assert_border_offset(block, 0, 1, 1, 0);
    }

    #[test]
    fn bottom_left() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::BOTTOM.union(Borders::LEFT));
        assert_border_offset(block, 1, 0, 0, 1);
    }

    #[test]
    fn full_layout() {
        let block: Block<(), Empty> = Block::with_child(Empty::default());
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 10,
            max_height: 10,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 10,
                height: 10,
            },
            "the block should have dimensions (10, 10)."
        );

        assert_eq!(
            layout_node.children[0].rect,
            Rect {
                x: 1,
                y: 1,
                width: 8,
                height: 8
            },
            "the only child should have an offset of (1, 1), and dimensions (8, 8)"
        );
    }

    #[test]
    fn vertical_layout() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::BOTTOM.union(Borders::TOP));
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 10,
            max_height: 10,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 10,
                height: 10,
            },
            "the block should have dimensions (10, 10)."
        );

        assert_eq!(
            layout_node.children[0].rect,
            Rect {
                x: 0,
                y: 1,
                width: 10,
                height: 8
            },
            "the only child should have an offset of (0, 1), and dimensions (10, 8)"
        );
    }

    #[test]
    fn horizontal_layout() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::LEFT.union(Borders::RIGHT));
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 10,
            max_height: 10,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 10,
                height: 10,
            },
            "the block should have dimensions (10, 10)."
        );

        assert_eq!(
            layout_node.children[0].rect,
            Rect {
                x: 1,
                y: 0,
                width: 8,
                height: 10
            },
            "the only child should have an offset of (1, 0), and dimensions (8, 10)"
        );
    }

    #[test]
    fn irregular_layout_one() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::LEFT.union(Borders::TOP));
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 10,
            max_height: 10,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 10,
                height: 10,
            },
            "the block should have dimensions (10, 10)."
        );

        assert_eq!(
            layout_node.children[0].rect,
            Rect {
                x: 1,
                y: 1,
                width: 9,
                height: 9
            },
            "the only child should have an offset of (1, 1), and dimensions (9, 9)"
        );
    }

    #[test]
    fn irregular_layout_two() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::BOTTOM.union(Borders::RIGHT));
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 10,
            max_height: 10,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 10,
                height: 10,
            },
            "the block should have dimensions (10, 10)."
        );

        assert_eq!(
            layout_node.children[0].rect,
            Rect {
                x: 0,
                y: 0,
                width: 9,
                height: 9
            },
            "the only child should have an offset of (0, 0), and dimensions (9, 9)"
        );
    }

    #[test]
    fn irregular_layout_three() {
        let block: Block<(), Empty> =
            Block::with_child(Empty::default()).borders(Borders::RIGHT.union(Borders::TOP));
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 10,
            max_height: 10,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 10,
                height: 10,
            },
            "the block should have dimensions (10, 10)."
        );

        assert_eq!(
            layout_node.children[0].rect,
            Rect {
                x: 0,
                y: 1,
                width: 9,
                height: 9
            },
            "the only child should have an offset of (0, 1), and dimensions (9, 9)"
        );
    }

    #[test]
    fn too_small_layout() {
        let block: Block<(), Empty> = Block::with_child(Empty::default());
        let mut layout_node = LayoutNode::default();
        let bounds = Bounds {
            min_width: 0,
            min_height: 0,
            max_width: 2,
            max_height: 2,
        };

        assert_eq!(
            block.layout(bounds, &mut layout_node),
            Size {
                width: 0,
                height: 0,
            },
            "the area should be 0"
        );

        assert_eq!(layout_node.children.len(), 0, "layout node should be empty");
    }
}
