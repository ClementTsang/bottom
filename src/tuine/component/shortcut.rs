use super::Component;

/// A [`Component`] to handle keyboard shortcuts and assign actions to them.
///
/// Inspired by [Flutter's approach](https://docs.flutter.dev/development/ui/advanced/actions_and_shortcuts).
pub struct Shortcut<Msg: 'static> {
    _p: std::marker::PhantomData<Msg>,
}

impl<Msg> Component for Shortcut<Msg> {
    type Message = Msg;

    type Properties = ();

    fn on_event(
        &mut self, bounds: tui::layout::Rect, event: crate::tuine::Event,
        messages: &mut Vec<Self::Message>,
    ) -> crate::tuine::Status {
        crate::tuine::Status::Ignored
    }

    fn update(&mut self, message: Self::Message) -> super::ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> super::ShouldRender {
        false
    }

    fn draw<B: tui::backend::Backend>(
        &mut self, bounds: tui::layout::Rect, frame: &mut tui::Frame<'_, B>,
    ) {
        todo!()
    }
}
