use std::cmp::min;

use crossterm::event::{KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use fxhash::FxHashMap;
use itertools::{EitherOrBoth, Itertools};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    text::{Span, Spans},
    widgets::{Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{
        event::{ComponentEventResult, MultiKey, MultiKeyResult},
        widgets::tui_stuff::BlockBuilder,
        Component,
    },
    canvas::Painter,
    constants::*,
};

pub struct HelpDialog {
    current_index: usize,
    max_index: usize,
    bounds: Rect,
    wrapped_text: Vec<Vec<Spans<'static>>>,
    left_column_width: Constraint,
    right_column_width: Constraint,

    /// Manages the `gg` double-tap shortcut.
    gg_manager: MultiKey,

    /// A jury-rigged solution for shortcut indices.
    /// TODO: [Refactor] Shortcut indices system - THIS DOES NOT SCALE WELL IN THE FUTURE! Write a better system like multikey (but for multiple combos).
    shortcut_indices: FxHashMap<u32, usize>,
}

impl Default for HelpDialog {
    fn default() -> Self {
        Self {
            current_index: Default::default(),
            max_index: Default::default(),
            bounds: Default::default(),
            left_column_width: Constraint::Length(0),
            right_column_width: Constraint::Length(0),
            wrapped_text: Default::default(),
            gg_manager: MultiKey::register(vec!['g', 'g']),
            shortcut_indices: FxHashMap::default(),
        }
    }
}

impl HelpDialog {
    pub fn rebuild_wrapped_text(&mut self, painter: &Painter) {
        let left_column_width = HELP_TEXT
            .iter()
            .map(|text| {
                text.iter()
                    .map(|[labels, _details]| {
                        labels
                            .lines()
                            .map(|line| UnicodeWidthStr::width(line))
                            .max()
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
            + 2;
        let right_column_width = (self.bounds.width as usize).saturating_sub(left_column_width);

        self.left_column_width = Constraint::Length(left_column_width as u16);
        self.right_column_width = Constraint::Length(right_column_width as u16);

        let mut shortcut_index = 1;
        let mut current_index = HELP_TITLES.len() + 2;

        // let mut section_indices: Vec<usize> = Vec::with_capacity(HELP_TITLES.len());
        // let mut help_title_index = HELP_CONTENTS_TEXT.len() + 1;
        // Behold, this monstrosity of an iterator (I'm sorry).
        // Be warned, for when you stare into the iterator, the iterator stares back.

        let wrapped_details_iter = HELP_TEXT.iter().map(|text| {
            text.iter()
                .map(|[labels, details]| {
                    let labels = textwrap::fill(labels, left_column_width);
                    let details = textwrap::fill(details, right_column_width);

                    labels
                        .lines()
                        .zip_longest(details.lines())
                        .map(|z| match z {
                            EitherOrBoth::Both(a, b) => vec![
                                Spans::from(Span::styled(
                                    a.to_string(),
                                    painter.colours.text_style,
                                )),
                                Spans::from(Span::styled(
                                    b.to_string(),
                                    painter.colours.text_style,
                                )),
                            ],
                            EitherOrBoth::Left(s) => {
                                vec![Spans::from(Span::styled(
                                    s.to_string(),
                                    painter.colours.text_style,
                                ))]
                            }
                            EitherOrBoth::Right(s) => vec![
                                Spans::default(),
                                Spans::from(Span::styled(
                                    s.to_string(),
                                    painter.colours.text_style,
                                )),
                            ],
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>()
        });

        self.wrapped_text = HELP_CONTENTS_TEXT
            .iter()
            .map(|t| vec![Spans::from(Span::styled(*t, painter.colours.text_style))])
            .chain(
                HELP_TITLES
                    .iter()
                    .zip(wrapped_details_iter)
                    .map(|(title, text)| {
                        self.shortcut_indices.insert(shortcut_index, current_index);
                        shortcut_index += 1;
                        current_index += 2 + text.len();
                        std::iter::once(vec![Spans::default()])
                            .chain(std::iter::once(vec![Spans::from(Span::styled(
                                *title,
                                painter.colours.highlighted_border_style,
                            ))]))
                            .chain(text)
                    })
                    .flatten(),
            )
            .collect();

        self.max_index = self
            .wrapped_text
            .len()
            .saturating_sub(self.bounds.height as usize);

        for value in self.shortcut_indices.values_mut() {
            *value = min(*value, self.max_index);
        }

        if self.current_index > self.max_index {
            self.current_index = self.max_index;
        }
    }

    pub fn draw_help<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, block_area: Rect,
    ) {
        let block = BlockBuilder::new("Help")
            .borders(Borders::all())
            .show_esc(true)
            .build(painter, block_area);

        let inner_area = block.inner(block_area);
        if inner_area != self.bounds {
            self.bounds = inner_area;
            self.rebuild_wrapped_text(painter);
        }
        let end_index = self.current_index + inner_area.height as usize;

        let split_area = Layout::default()
            .constraints(vec![Constraint::Length(1); inner_area.height as usize])
            .direction(tui::layout::Direction::Vertical)
            .split(inner_area);

        self.wrapped_text[self.current_index..end_index]
            .iter()
            .zip(split_area)
            .for_each(|(row, area)| {
                if row.len() > 1 {
                    let row_split_area = Layout::default()
                        .constraints(vec![self.left_column_width, self.right_column_width])
                        .direction(tui::layout::Direction::Horizontal)
                        .split(area);

                    let left_area = row_split_area[0];
                    let right_area = row_split_area[1];

                    f.render_widget(Paragraph::new(row[0].clone()), left_area);
                    f.render_widget(Paragraph::new(row[1].clone()), right_area);
                } else if let Some(line) = row.get(0) {
                    f.render_widget(Paragraph::new(line.clone()), area);
                }
            });

        f.render_widget(block, block_area);
    }

    fn move_up(&mut self, amount: usize) -> ComponentEventResult {
        let new_index = self.current_index.saturating_sub(amount);
        if self.current_index == new_index {
            ComponentEventResult::NoRedraw
        } else {
            self.current_index = new_index;
            ComponentEventResult::Redraw
        }
    }

    fn move_down(&mut self, amount: usize) -> ComponentEventResult {
        let new_index = self.current_index + amount;
        if new_index > self.max_index || self.current_index == new_index {
            ComponentEventResult::NoRedraw
        } else {
            self.current_index = new_index;
            ComponentEventResult::Redraw
        }
    }

    fn skip_to_first(&mut self) -> ComponentEventResult {
        if self.current_index == 0 {
            ComponentEventResult::NoRedraw
        } else {
            self.current_index = 0;
            ComponentEventResult::Redraw
        }
    }

    fn skip_to_last(&mut self) -> ComponentEventResult {
        if self.current_index == self.max_index {
            ComponentEventResult::NoRedraw
        } else {
            self.current_index = self.max_index;
            ComponentEventResult::Redraw
        }
    }
}

impl Component for HelpDialog {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        use crossterm::event::KeyCode::{Char, Down, Up};

        if event.modifiers == KeyModifiers::NONE || event.modifiers == KeyModifiers::SHIFT {
            match event.code {
                Down if event.modifiers == KeyModifiers::NONE => self.move_down(1),
                Up if event.modifiers == KeyModifiers::NONE => self.move_up(1),
                Char(c) => match c {
                    'j' => self.move_down(1),
                    'k' => self.move_up(1),
                    'g' => match self.gg_manager.input('g') {
                        MultiKeyResult::Completed => self.skip_to_first(),
                        MultiKeyResult::Accepted | MultiKeyResult::Rejected => {
                            ComponentEventResult::NoRedraw
                        }
                    },
                    'G' => self.skip_to_last(),
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                        if let Some(potential_index) = c.to_digit(10) {
                            if let Some(&new_index) = self.shortcut_indices.get(&potential_index) {
                                if new_index != self.current_index {
                                    self.current_index = new_index;
                                    ComponentEventResult::Redraw
                                } else {
                                    ComponentEventResult::NoRedraw
                                }
                            } else {
                                ComponentEventResult::Unhandled
                            }
                        } else {
                            ComponentEventResult::Unhandled
                        }
                    }
                    _ => ComponentEventResult::Unhandled,
                },
                _ => ComponentEventResult::Unhandled,
            }
        } else {
            ComponentEventResult::Unhandled
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        match event.kind {
            MouseEventKind::ScrollDown => self.move_down(1),
            MouseEventKind::ScrollUp => self.move_up(1),
            _ => ComponentEventResult::Unhandled,
        }
    }
}
