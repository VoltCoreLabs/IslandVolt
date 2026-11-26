mod presentation;

use crate::{island::Island, volume_slider};
use gpui::{Context, Pixels, ScrollHandle, Task};
use islandvolt_platform_windows::mixer::{self, Action, Session, Snapshot};
use std::time::{Duration, Instant};

pub(crate) struct MixerPanel {
    monitor: mixer::Mixer,
    _updates: Task<()>,
    snapshot: Option<Snapshot>,
    selection: Option<u64>,
    drag: Option<(u64, Instant)>,
    scroll: ScrollHandle,
    command_error: Option<&'static str>,
}

impl MixerPanel {
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub(crate) fn ready(&self) -> bool {
        self.snapshot.is_some()
    }

    fn rows(&self) -> &[Session] {
        self.snapshot
            .as_ref()
            .map_or(&[], |state| state.sessions.as_slice())
    }

    fn queue(&mut self, id: u64, action: Action) {
        self.command_error = self.monitor.adjust(id, action).err();
    }
}

impl Island {
    pub(crate) fn open_mixer(&mut self, cx: &mut Context<Self>) {
        if self.mixer.is_some() {
            return;
        }
        let (monitor, updates) = mixer::open();
        let task = cx.spawn(async move |this, cx| {
            while let Ok(snapshot) = updates.recv().await {
                if this
                    .update(cx, |island, cx| {
                        let Some(panel) = &mut island.mixer else {
                            return;
                        };
                        let has_session = |id| snapshot.sessions.iter().any(|row| row.id == id);
                        if panel.selection.is_none_or(|id| !has_session(id)) {
                            panel.selection = snapshot.sessions.first().map(|row| row.id);
                        }
                        let ended_drag = panel.drag.is_some_and(|(id, _)| !has_session(id));
                        if ended_drag {
                            panel.drag = None;
                        }
                        panel.snapshot = Some(snapshot);
                        if ended_drag && !island.hover.inside {
                            island.hover_changed(false, cx);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        self.show_sources = false;
        self.mixer = Some(MixerPanel {
            monitor,
            _updates: task,
            snapshot: None,
            selection: None,
            drag: None,
            scroll: ScrollHandle::new(),
            command_error: None,
        });
        self.sync_pet(cx);
        cx.notify();
    }

    pub(crate) fn close_mixer(&mut self, cx: &mut Context<Self>) {
        if self.mixer.take().is_some() {
            self.sync_pet(cx);
            cx.notify();
        }
    }

    fn move_mixer_slider(&mut self, x: Pixels, finished: bool, cx: &mut Context<Self>) {
        let Some(panel) = &mut self.mixer else {
            return;
        };
        let Some((id, last)) = panel.drag else {
            return;
        };
        if !finished && last.elapsed() < Duration::from_millis(50) {
            return;
        }
        panel.drag = (!finished).then(|| (id, Instant::now()));
        panel.queue(
            id,
            Action::Set(volume_slider::slider_percent(f32::from(x), self.wide)),
        );
        if finished && !self.hover.inside {
            self.hover_changed(false, cx);
        }
        cx.notify();
    }

    pub(crate) fn mixer_key(&mut self, key: &str, cx: &mut Context<Self>) -> bool {
        let Some(panel) = &mut self.mixer else {
            return false;
        };
        match key {
            "escape" => {
                let interrupted_drag = panel.dragging();
                self.close_mixer(cx);
                if interrupted_drag && !self.hover.inside {
                    self.hover_changed(false, cx);
                }
            }
            "up" | "down" => {
                if let Some(current) = panel
                    .rows()
                    .iter()
                    .position(|row| Some(row.id) == panel.selection)
                {
                    let next = if key == "up" {
                        current.saturating_sub(1)
                    } else {
                        (current + 1).min(panel.rows().len() - 1)
                    };
                    panel.selection = Some(panel.rows()[next].id);
                    panel.scroll.scroll_to_item(next);
                }
            }
            "left" | "right" | "m" => {
                if let Some(id) = panel.selection {
                    let action = match key {
                        "left" => Action::Step(-5),
                        "right" => Action::Step(5),
                        _ => Action::ToggleMute,
                    };
                    panel.queue(id, action);
                }
            }
            _ => return false,
        }
        cx.notify();
        true
    }
}
