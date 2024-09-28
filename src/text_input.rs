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

pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub content: Vec<SharedString>,
    pub content_idx: usize,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<ShapedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
    pub is_selecting: bool,
}

impl TextInput {
    pub fn left(&mut self, _: &Left, cx: &mut ViewContext<Self>) {
        if self.content_idx > 0 && self.cursor_offset() == 0 {
            self.move_up(cx);
            self.cursor_to_end(cx);
        } else if self.selected_range.is_empty() {
            self.move_x(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_x(self.selected_range.start, cx)
        }
    }

    pub fn right(&mut self, _: &Right, cx: &mut ViewContext<Self>) {
        if self.content_idx < self.content.len() - 1
            && self.cursor_offset() == self.content[self.content_idx].len()
        {
            self.move_down(cx);
            self.cursor_to_start(cx);
        } else if self.selected_range.is_empty() {
            self.move_x(self.next_boundary(self.cursor_offset()), cx);
        } else {
            self.move_x(self.selected_range.end, cx)
        }
    }

    pub fn up(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        if self.content_idx > 0 {
            self.move_up(cx);
            if self.content[self.content_idx].len() < self.cursor_offset() {
                self.cursor_to_end(cx);
            }
        }
    }

    pub fn down(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
        if self.content.len() - 1 > self.content_idx {
            self.move_down(cx);
            if self.content[self.content_idx].len() < self.cursor_offset() {
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
        if self.selected_range.is_empty() {
            if self.cursor_offset() == 0 && self.content_idx > 0 {
                let current_content = self.content[self.content_idx].clone();
                let previous_content = self.content[self.content_idx - 1].clone();

                self.move_x(previous_content.len(), cx);

                let merged_content = previous_content.to_string() + &current_content;
                self.content[self.content_idx - 1] = merged_content.into();

                self.content.remove(self.content_idx);

                self.move_up(cx)
            }
            self.select_to(
                self.previous_boundary(self.cursor_offset()),
                self.content_idx,
                cx,
            );
        }

        self.replace_text_in_range(None, "", cx);
    }

    pub fn delete(&mut self, _: &Delete, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(
                self.next_boundary(self.cursor_offset()),
                self.content_idx,
                cx,
            )
        }
        self.replace_text_in_range(None, "", cx)
    }

    pub fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut ViewContext<Self>) {
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(
                self.index_for_mouse_position(event.position).0,
                self.index_for_mouse_position(event.position).1,
                cx,
            );
        } else {
            self.move_x(self.index_for_mouse_position(event.position).0, cx)
        }
    }

    pub fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut ViewContext<Self>) {
        self.is_selecting = false;
    }

    pub fn on_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut ViewContext<Self>) {
        if self.is_selecting {
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
        let leftovers = if self.content[self.content_idx].len() > 0 {
            self.content[self.content_idx]
                [self.cursor_offset()..self.content[self.content_idx].len()]
                .to_string()
        } else {
            "".to_string()
        };

        let new_content = self.content[self.content_idx][..self.cursor_offset()].to_string();
        self.content[self.content_idx] = new_content.into();
        self.new_line(leftovers, cx);

        self.move_down(cx);
        self.cursor_to_start(cx);
    }

    pub fn new_line(&mut self, data: String, _cx: &mut ViewContext<Self>) {
        self.content.insert(self.content_idx + 1, data.into());
    }

    fn move_x(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        self.selected_range = offset..offset;
        cx.notify()
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
        let length = self.content[self.content_idx].len();
        self.move_x(length, cx);
    }

    pub fn cursor_to_start(&mut self, cx: &mut ViewContext<Self>) {
        self.move_x(0, cx);
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> (usize, usize) {
        if self.content[self.content_idx].is_empty() {
            return (0, self.content_idx);
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return (0, self.content_idx);
        };
        if position.y < bounds.top() {
            return (0, self.content_idx);
        }
        if position.y > bounds.bottom() {
            return (self.content.len(), self.content_idx);
        }
        (
            line.closest_index_for_x(position.x - bounds.left()),
            self.content_idx,
        )
    }

    fn select_to(&mut self, x_offset: usize, _y_offset: usize, cx: &mut ViewContext<Self>) {
        if self.selection_reversed {
            self.selected_range.start = x_offset
        } else {
            self.selected_range.end = x_offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content[self.content_idx].chars() {
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

        for ch in self.content[self.content_idx].chars() {
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
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content[self.content_idx]
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content[self.content_idx].len())
    }

    fn add_word_to_start_of_line(&mut self, word: &str, line: usize, cx: &mut ViewContext<Self>) {
        if self.content.len() <= line {
            self.new_line("".into(), cx);
        }

        let new_content = word.to_owned() + " " + &self.content[line];

        self.content[line] = new_content.into();
    }

    fn replace_text_in_range_without_moving(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content[self.content_idx] = (self.content[self.content_idx][0..range.start]
            .to_owned()
            + new_text
            + &self.content[self.content_idx][range.end..])
            .into();
        self.marked_range.take();
        cx.notify();
    }

    fn check_bounds(&mut self, cx: &mut ViewContext<Self>) {
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return;
        };

        let pixels = line.x_for_index(self.content[self.content_idx].len()) + bounds.left();
        let width = cx.window_bounds().get_bounds().right()
            - cx.window_bounds().get_bounds().left()
            - bounds.right();

        if pixels > width {
            let content_string = self.content[self.content_idx].to_string();
            let content = content_string.split(" ");

            if content.clone().count() > 1 {
                let last_word = content.last().unwrap();

                self.add_word_to_start_of_line(last_word, self.content_idx + 1, cx);
                self.replace_text_in_range_without_moving(
                    Some(content_string.len() - last_word.len() - 1..content_string.len()),
                    "",
                    cx,
                );

                if self.selected_range.start >= content_string.len() - last_word.len() {
                    self.content_idx += 1;
                    self.selected_range = last_word.len()..last_word.len();
                }
                return;
            }
            self.enter(&Enter, cx)
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
        Some(self.content[self.content_idx][range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _cx: &mut ViewContext<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _cx: &mut ViewContext<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _cx: &mut ViewContext<Self>) {
        self.marked_range = None;
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
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content[self.content_idx] = (self.content[self.content_idx][0..range.start]
            .to_owned()
            + new_text
            + &self.content[self.content_idx][range.end..])
            .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();

        self.check_bounds(cx);

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
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content[self.content_idx] = (self.content[self.content_idx][0..range.start]
            .to_owned()
            + new_text
            + &self.content[self.content_idx][range.end..])
            .into();
        self.marked_range = Some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
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
        let last_layout = self.last_layout.as_ref()?;
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
            .p(px(40.))
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
            .children(self.content.iter().enumerate().map(|(i, _)| {
                div().pt(px(20. * i as f32)).child(TextElement {
                    // XXX TODO no need to pass whole view each time
                    input: cx.view().clone(),
                    index: i,
                })
            }))
    }
}
