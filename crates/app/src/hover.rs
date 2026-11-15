use crate::island::Island;
use gpui::{Context, Task};

pub struct Hover {
    pub inside: bool,
    pub pending: Option<Task<()>>,
}

impl Island {
    pub(crate) fn hover_changed(&mut self, inside: bool, cx: &mut Context<Self>) {
        self.hover.inside = inside;
        self.hover.pending.take();
        if inside {
            self.expanded = true;
            self.transition.set(true);
            cx.notify();
        } else if self.can_auto_collapse() {
            let timer = cx
                .background_executor()
                .timer(std::time::Duration::from_millis(300));
            self.hover.pending = Some(cx.spawn(async move |this, cx| {
                timer.await;
                let _ = this.update(cx, |island, cx| {
                    if island.can_auto_collapse() {
                        island.expanded = false;
                        island.transition.set(false);
                        cx.notify();
                    }
                });
            }));
        }
    }

    fn can_auto_collapse(&self) -> bool {
        can_collapse(
            self.hover.inside,
            self.pinned,
            self.dragging_volume,
            self.keyboard.open,
        )
    }
}

fn can_collapse(inside: bool, pinned: bool, dragging: bool, keyboard: bool) -> bool {
    !inside && !pinned && !dragging && !keyboard
}

#[cfg(test)]
mod tests {
    use super::can_collapse;

    #[test]
    fn keyboard_pin_and_drag_independently_hold_panel_open() {
        assert!(can_collapse(false, false, false, false));
        for inside in [false, true] {
            for pinned in [false, true] {
                for dragging in [false, true] {
                    assert!(!can_collapse(inside, pinned, dragging, true));
                }
            }
        }
        assert!(!can_collapse(true, false, false, false));
        assert!(!can_collapse(false, true, false, false));
        assert!(!can_collapse(false, false, true, false));
    }
}
