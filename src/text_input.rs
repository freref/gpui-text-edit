use crate::text_element::TextElement;
use gpui::*;
use std::ops::Range;
use unicode_segmentation::*;

actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Enter,
        Up,
        Down
    ]
);

pub struct TextLine {
    pub content: SharedString,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<ShapedLine>,
    pub is_selecting: bool,
}

pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub content: Vec<TextLine>,
    pub content_idx: usize,
    pub last_bounds: Option<Bounds<Pixels>>,
}

impl TextInput {
    pub fn left(&mut self, _: &Left, cx: &mut ViewContext<Self>) {
        if self.content_idx > 0 && self.cursor_offset() == 0 {
            self.move_up(cx);
            self.cursor_to_end(cx);
        } else if self.content[self.content_idx].selected_range.is_empty() {
            self.move_x(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_x(self.content[self.content_idx].selected_range.start, cx)
        }
    }

    pub fn right(&mut self, _: &Right, cx: &mut ViewContext<Self>) {
        if self.content_idx < self.content.len() - 1
            && self.cursor_offset() == self.content[self.content_idx].content.len()
        {
            self.move_down(cx);
            self.cursor_to_start(cx);
        } else if self.content[self.content_idx].selected_range.is_empty() {
            self.move_x(self.next_boundary(self.cursor_offset()), cx);
        } else {
            self.move_x(self.content[self.content_idx].selected_range.end, cx)
        }
    }

    pub fn up(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        if self.content_idx > 0 {
            self.move_up(cx);
            if self.content[self.content_idx].content.len() < self.cursor_offset() {
                self.cursor_to_end(cx);
            }
        }
    }

    pub fn down(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
        if self.content.len() - 1 > self.content_idx {
            self.move_down(cx);
            if self.content[self.content_idx].content.len() < self.cursor_offset() {
                self.cursor_to_end(cx);
            }
        }
    }

    pub fn select_left(&mut self, _: &SelectLeft, cx: &mut ViewContext<Self>) {
        self.select_to(
            self.previous_boundary(self.cursor_offset()),
            self.content_idx,
            cx,
        );
    }

    pub fn select_right(&mut self, _: &SelectRight, cx: &mut ViewContext<Self>) {
        self.select_to(
            self.next_boundary(self.cursor_offset()),
            self.content_idx,
            cx,
        );
    }

    pub fn select_all(&mut self, _: &SelectAll, cx: &mut ViewContext<Self>) {
        self.move_x(0, cx);
        self.select_to(self.content.len(), self.content_idx, cx)
    }

    pub fn home(&mut self, _: &Home, cx: &mut ViewContext<Self>) {
        self.move_x(0, cx);
    }

    pub fn end(&mut self, _: &End, cx: &mut ViewContext<Self>) {
        self.move_x(self.content.len(), cx);
    }

    pub fn backspace(&mut self, _: &Backspace, cx: &mut ViewContext<Self>) {
        if self.content[self.content_idx].selected_range.is_empty() {
            if self.cursor_offset() == 0 && self.content_idx > 0 {
                let current_content = self.content[self.content_idx].content.clone();
                let previous_content = self.content[self.content_idx - 1].content.clone();

                self.move_x(previous_content.len(), cx);

                let merged_content = previous_content.to_string() + &current_content;
                self.content[self.content_idx - 1].content = merged_content.into();

                self.content.remove(self.content_idx);

                self.move_up(cx);
                let content_len = self.content[self.content_idx].content.len();
                self.content[self.content_idx].selected_range = content_len..content_len;
            } else {
                self.select_to(
                    self.previous_boundary(self.cursor_offset()),
                    self.content_idx,
                    cx,
                );
            }
        }

        self.replace_text_in_range(None, "", cx);
    }

    pub fn delete(&mut self, _: &Delete, cx: &mut ViewContext<Self>) {
        if self.content[self.content_idx].selected_range.is_empty() {
            self.select_to(
                self.next_boundary(self.cursor_offset()),
                self.content_idx,
                cx,
            )
        }
        self.replace_text_in_range(None, "", cx)
    }

    pub fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut ViewContext<Self>) {
        self.content[self.content_idx].is_selecting = true;

        if event.modifiers.shift {
            self.select_to(
                self.index_for_mouse_position(event.position).0,
                self.index_for_mouse_position(event.position).1,
                cx,
            );
        } else {
            let pos = self.index_for_mouse_position(event.position);
            self.move_x(pos.0, cx);
            self.move_y(pos.1, cx);
        }
    }

    pub fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut ViewContext<Self>) {
        self.content[self.content_idx].is_selecting = false;
    }

    pub fn on_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut ViewContext<Self>) {
        if self.content[self.content_idx].is_selecting {
            self.select_to(
                self.index_for_mouse_position(event.position).0,
                self.index_for_mouse_position(event.position).1,
                cx,
            );
        }
    }

    pub fn show_character_palette(&mut self, _: &ShowCharacterPalette, cx: &mut ViewContext<Self>) {
        cx.show_character_palette();
    }

    pub fn enter(&mut self, _: &Enter, cx: &mut ViewContext<Self>) {
        let leftovers = if self.content[self.content_idx].content.len() > 0 {
            self.content[self.content_idx].content
                [self.cursor_offset()..self.content[self.content_idx].content.len()]
                .to_string()
        } else {
            "".to_string()
        };

        let new_content =
            self.content[self.content_idx].content[..self.cursor_offset()].to_string();
        self.content[self.content_idx].content = new_content.into();
        self.new_line(leftovers, self.content_idx + 1, cx);

        self.move_down(cx);
        self.cursor_to_start(cx);
    }

    pub fn new_line(&mut self, data: String, index: usize, _cx: &mut ViewContext<Self>) {
        self.content.insert(
            index,
            TextLine {
                content: data.into(),
                selected_range: 0..0,
                selection_reversed: false,
                marked_range: None,
                last_layout: None,
                is_selecting: false,
            },
        );
    }

    fn move_x(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        self.content[self.content_idx].selected_range = offset..offset;
        cx.notify()
    }

    fn move_y(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        self.content_idx = offset;
        cx.notify();
    }

    fn move_up(&mut self, cx: &mut ViewContext<Self>) {
        self.content_idx -= 1;
        cx.notify();
    }

    fn move_down(&mut self, cx: &mut ViewContext<Self>) {
        self.content_idx += 1;
        cx.notify();
    }

    pub fn cursor_to_end(&mut self, cx: &mut ViewContext<Self>) {
        let length = self.content[self.content_idx].content.len();
        self.move_x(length, cx);
    }

    pub fn cursor_to_start(&mut self, cx: &mut ViewContext<Self>) {
        self.move_x(0, cx);
    }

    pub fn cursor_offset(&self) -> usize {
        if self.content[self.content_idx].selection_reversed {
            self.content[self.content_idx].selected_range.start
        } else {
            self.content[self.content_idx].selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> (usize, usize) {
        let mut y = ((position.y.0 + 4.) / 14. - 1.).floor() as usize;
        y = y.min(self.content.len() - 1);

        let (Some(bounds), Some(line)) = (
            self.last_bounds.as_ref(),
            self.content[self.content_idx].last_layout.as_ref(),
        ) else {
            return (0, y);
        };

        let mut x = line.closest_index_for_x(position.x - bounds.left());
        x = x.min(self.content[y].content.len());
        (x, y)
    }

    fn select_to(&mut self, x_offset: usize, _y_offset: usize, cx: &mut ViewContext<Self>) {
        if self.content[self.content_idx].selection_reversed {
            self.content[self.content_idx].selected_range.start = x_offset
        } else {
            self.content[self.content_idx].selected_range.end = x_offset
        };
        if self.content[self.content_idx].selected_range.end
            < self.content[self.content_idx].selected_range.start
        {
            self.content[self.content_idx].selection_reversed =
                !self.content[self.content_idx].selection_reversed;
            self.content[self.content_idx].selected_range =
                self.content[self.content_idx].selected_range.end
                    ..self.content[self.content_idx].selected_range.start;
        }
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content[self.content_idx].content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content[self.content_idx].content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content[self.content_idx]
            .content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content[self.content_idx]
            .content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content[self.content_idx].content.len())
    }

    fn add_word_to_start_of_line(&mut self, word: &str, index: usize, cx: &mut ViewContext<Self>) {
        if self.content.len() <= index {
            self.new_line("".into(), index, cx);
        }

        let new_content = if self.content[index].content.len() > 0 {
            word.to_owned() + " " + &self.content[index].content
        } else {
            word.to_owned() + &self.content[index].content
        };

        self.content[index].content = new_content.into();

        cx.notify();
    }

    fn replace_text_in_range_without_moving(
        &mut self,
        range_utf16: Option<Range<usize>>,
        index: usize,
        new_text: &str,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.content[index].marked_range.clone())
            .unwrap_or(self.content[index].selected_range.clone());

        self.content[index].content = (self.content[index].content[0..range.start].to_owned()
            + new_text
            + &self.content[index].content[range.end..])
            .into();
        self.content[index].marked_range.take();
        cx.notify();
    }

    pub fn update_layout(&mut self, index: usize, cx: &mut ViewContext<Self>) {
        let content = self.content[index].content.clone();
        let style = cx.text_style();

        let (display_text, text_color) = (content.clone(), style.color);

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if let Some(marked_range) = self.content[index].marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run.clone()
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(cx.rem_size());
        let line = cx
            .text_system()
            .shape_line(display_text, font_size, &runs)
            .unwrap();

        self.content[index].last_layout = Some(line);
    }

    fn check_bounds(&mut self, index: usize, cx: &mut ViewContext<Self>) {
        let (Some(bounds), Some(layout)) = (
            self.last_bounds.as_ref(),
            self.content[index].last_layout.as_ref(),
        ) else {
            return;
        };

        let pixels =
            layout.x_for_index(self.content[index].content.len()) + bounds.left() + bounds.right();
        let width = cx.window_bounds().get_bounds().right()
            - cx.window_bounds().get_bounds().left()
            - bounds.right()
            - bounds.left();

        if pixels >= width {
            let content_string = self.content[index].content.to_string();
            let mut last_index = layout.closest_index_for_x(width);

            while last_index > 0 && !content_string[last_index..].starts_with(' ') {
                last_index -= 1;
            }

            if last_index == 0 {
                self.enter(&Enter, cx);
                return;
            }

            if last_index > 0 {
                last_index += 1;
            }

            let len = content_string.len() - last_index;
            let leftovers = &content_string[last_index..content_string.len()];

            println!("{}", leftovers);

            self.add_word_to_start_of_line(leftovers, index + 1, cx);
            self.replace_text_in_range_without_moving(
                Some(last_index - 1..content_string.len()),
                index,
                "",
                cx,
            );

            if self.content[index].selected_range.start >= content_string.len() - len
                && index == self.content_idx
            {
                self.content_idx += 1;
                let pos = len;
                self.content[self.content_idx].selected_range = pos..pos;
            }

            self.check_bounds(index + 1, cx);
        }
    }
}

impl FocusableView for TextInput {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ViewInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        Some(self.content[self.content_idx].content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _cx: &mut ViewContext<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.content[self.content_idx].selected_range),
            reversed: self.content[self.content_idx].selection_reversed,
        })
    }

    fn marked_text_range(&self, _cx: &mut ViewContext<Self>) -> Option<Range<usize>> {
        self.content[self.content_idx]
            .marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _cx: &mut ViewContext<Self>) {
        self.content[self.content_idx].marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.content[self.content_idx].marked_range.clone())
            .unwrap_or(self.content[self.content_idx].selected_range.clone());

        self.content[self.content_idx].content =
            (self.content[self.content_idx].content[0..range.start].to_owned()
                + new_text
                + &self.content[self.content_idx].content[range.end..])
                .into();
        self.content[self.content_idx].selected_range =
            range.start + new_text.len()..range.start + new_text.len();
        self.content[self.content_idx].marked_range.take();

        self.check_bounds(self.content_idx, cx);

        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.content[self.content_idx].marked_range.clone())
            .unwrap_or(self.content[self.content_idx].selected_range.clone());

        self.content[self.content_idx].content =
            (self.content[self.content_idx].content[0..range.start].to_owned()
                + new_text
                + &self.content[self.content_idx].content[range.end..])
                .into();
        self.content[self.content_idx].marked_range =
            Some(range.start..range.start + new_text.len());
        self.content[self.content_idx].selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.content[self.content_idx].last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }
}

impl Render for TextInput {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p(px(4.))
            .flex()
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .text_size(px(12.))
            .children((0..self.content.len()).map(|i| {
                div().pt(px(14. * i as f32)).child(TextElement {
                    input: cx.view().clone(),
                    index: i,
                })
            }))
    }
}
