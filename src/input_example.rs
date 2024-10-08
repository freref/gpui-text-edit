use crate::text_input::TextInput;
use gpui::*;

pub struct InputExample {
    pub text_input: View<TextInput>,
    pub focus_handle: FocusHandle,
}

impl FocusableView for InputExample {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for InputExample {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(self.text_input.clone())
            .size_full()
            .bg(white())
            .cursor(CursorStyle::IBeam)
            .h_full()
    }
}
