use crate::island::Island;
use gpui::{Context, KeyDownEvent, Task, Window};
use islandvolt_platform_windows::shortcut::{FocusRestore, Shortcut};
use islandvolt_platform_windows::volume::Action;

pub struct Keyboard {
    _shortcut: Option<Shortcut>,
    _task: Option<Task<()>>,
    pub open: bool,
    pub unavailable: bool,
    previous: Option<FocusRestore>,
}

impl Keyboard {
    pub fn new(window: &Window, cx: &mut Context<Island>) -> Self {
        let (shortcut, task) = match Shortcut::register() {
            Ok((shortcut, events)) => {
                let task = cx.spawn_in(window, async move |this, cx| {
                    while events.recv().await.is_ok() {
                        if this
                            .update_in(cx, |island, window, cx| {
                                island.toggle_keyboard(window, cx);
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                });
                (Some(shortcut), Some(task))
            }
            Err(error) => {
                eprintln!("Ctrl+Alt+Space недоступна; используйте наведение или трей: {error}");
                (None, None)
            }
        };
        Self {
            unavailable: shortcut.is_none(),
            _shortcut: shortcut,
            _task: task,
            open: false,
            previous: None,
        }
    }
}

impl Island {
    fn toggle_keyboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.keyboard.open && window.is_window_active() {
            self.dismiss_panel(window, cx);
            return;
        }
        if let Err(error) = islandvolt_platform_windows::set_visible(window, true) {
            eprintln!("Не удалось показать остров: {error}");
            return;
        }
        self.keyboard.previous = Some(FocusRestore::capture());
        self.keyboard.open = true;
        self.hidden = false;
        self.hover.pending.take();
        self.expanded = true;
        self.transition.set(true);
        window.activate_window();
        window.focus(&self.focus);
        cx.notify();
    }

    pub(crate) fn release_keyboard(&mut self, window: &Window) {
        self.keyboard.open = false;
        self.hover.pending.take();
        if let Some(previous) = self.keyboard.previous.take() {
            previous.restore_if_current(window);
        }
    }

    fn dismiss_panel(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.release_keyboard(window);
        self.pinned = false;
        self.settings.save_pinned(false);
        self.show_sources = false;
        self.dragging_volume = false;
        self.expanded = false;
        self.transition.set(false);
        cx.notify();
    }

    pub(crate) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        // Modified keystrokes belong to global shortcuts, not local playback controls.
        let modifiers = &event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" if self.show_sources => self.show_sources = false,
            "escape" => self.dismiss_panel(window, cx),
            "left" => {
                self.volume_error = self
                    .volume_monitor
                    .control(Action::Decrease)
                    .err()
                    .map(str::to_string)
            }
            "right" => {
                self.volume_error = self
                    .volume_monitor
                    .control(Action::Increase)
                    .err()
                    .map(str::to_string)
            }
            "m" => {
                self.volume_error = self
                    .volume_monitor
                    .control(Action::ToggleMute)
                    .err()
                    .map(str::to_string)
            }
            "space" => {
                if let Some(track) = &self.track {
                    self.media_error = self
                        .media_monitor
                        .toggle(track.session_id)
                        .err()
                        .map(str::to_string);
                }
            }
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}
