//! GPUI owner for the pet: one cancellable deadline, independent of island redraws.
use gpui::{Context, IntoElement, Render, Task, div, prelude::*, px, rgb, svg};
use islandvolt_features::pets::{Animator, Package};

pub struct Pet {
    animator: Option<Animator>,
    timer: Option<Task<()>>,
}

impl Pet {
    pub fn new(cx: &mut Context<Self>, visible: bool) -> Self {
        let mut animator =
            match Package::from_json(include_str!("../../../assets/pets/volt-cat/manifest.json")) {
                Ok(package) => Some(Animator::new(
                    package,
                    !islandvolt_platform_windows::animations_enabled(),
                )),
                Err(error) => {
                    eprintln!("Питомец отключён: {error}");
                    None
                }
            };
        if let Some(animator) = &mut animator {
            animator.set_visible(visible);
        }
        let mut pet = Self {
            animator,
            timer: None,
        };
        pet.arm(cx);
        pet
    }

    pub fn set_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        if self
            .animator
            .as_mut()
            .is_some_and(|animator| animator.set_visible(visible))
        {
            self.timer.take();
            self.arm(cx);
            cx.notify();
        }
    }

    pub fn react(&mut self, cx: &mut Context<Self>) {
        if let Some(animator) = &mut self.animator {
            animator.react();
        }
        self.arm(cx);
        cx.notify();
    }

    fn arm(&mut self, cx: &mut Context<Self>) {
        self.timer.take();
        let Some(delay) = self.animator.as_ref().and_then(Animator::next_delay) else {
            return;
        };
        let deadline = cx.background_executor().timer(delay);
        self.timer = Some(cx.spawn(async move |this, cx| {
            deadline.await;
            let _ = this.update(cx, |pet, cx| {
                if let Some(animator) = &mut pet.animator {
                    animator.advance();
                }
                pet.arm(cx);
                cx.notify();
            });
        }));
    }
}

impl Render for Pet {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("island-pet")
            .size(px(32.0))
            .flex_shrink_0()
            .cursor_pointer()
            .on_click(cx.listener(|pet, _, _, cx| {
                cx.stop_propagation();
                pet.react(cx);
            }))
            .when_some(self.animator.as_ref(), |view, animator| {
                view.child(
                    svg()
                        .path(animator.frame().asset.clone())
                        .size_full()
                        .text_color(rgb(0xf5ead8)),
                )
            })
    }
}

impl crate::island::Island {
    pub(crate) fn sync_pet(&self, cx: &mut Context<Self>) {
        let visible = !self.hidden && !self.pet_hidden && !self.show_sources;
        self.pet.update(cx, |pet, cx| pet.set_visible(visible, cx));
    }
}
