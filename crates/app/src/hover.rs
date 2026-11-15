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
        } else if !self.pinned && !self.dragging_volume {
            let timer = cx
                .background_executor()
                .timer(std::time::Duration::from_millis(300));
            self.hover.pending = Some(cx.spawn(async move |this, cx| {
                timer.await;
                let _ = this.update(cx, |island, cx| {
                    if !island.hover.inside && !island.pinned && !island.dragging_volume {
                        island.expanded = false;
                        island.transition.set(false);
                        cx.notify();
                    }
                });
            }));
        }
    }
}
