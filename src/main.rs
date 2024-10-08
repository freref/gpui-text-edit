mod input_example;
mod text_element;
mod text_input;

use gpui::*;
use input_example::InputExample;
use text_input::TextInput;
use text_input::TextLine;
use text_input::{
    Backspace, Delete, Down, End, Enter, Home, Left, Right, SelectAll, SelectLeft, SelectRight,
    ShowCharacterPalette, Up,
};

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.activate(true);
        cx.on_action(quit);
        cx.set_menus(vec![Menu {
            name: "set_menus".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);
        let bounds = Bounds::centered(None, size(px(500.0), px(500.0)), cx);
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
            KeyBinding::new("enter", Enter, None),
            KeyBinding::new("up", Up, None),
            KeyBinding::new("down", Down, None),
        ]);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |cx| {
                    let text_input = cx.new_view(|cx| TextInput {
                        focus_handle: cx.focus_handle(),
                        content: vec![TextLine {
                            content: "".into(),
                            selected_range: 0..0,
                            selection_reversed: false,
                            marked_range: None,
                            last_layout: None,
                            is_selecting: false,
                        }],
                        content_idx: 0,
                        last_bounds: None,
                    });
                    cx.new_view(|cx| InputExample {
                        text_input,
                        focus_handle: cx.focus_handle(),
                    })
                },
            )
            .unwrap();
        window
            .update(cx, |view, cx| {
                cx.focus_view(&view.text_input);
                cx.activate(true);
            })
            .unwrap();
    });
}

actions!(set_menus, [Quit]);

fn quit(_: &Quit, cx: &mut AppContext) {
    cx.quit();
}
