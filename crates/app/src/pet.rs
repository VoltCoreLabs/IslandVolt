//! GPUI owner for the pet: one cancellable deadline, independent of island redraws.
use crate::pet_catalog::{PetCatalog, PetKind};
use gpui::{Context, IntoElement, Render, Task, div, img, prelude::*, px};
use islandvolt_features::pets::{Animator, Package};

pub struct Pet {
    animator: Option<Animator>,
    catalog: PetCatalog,
    kind: PetKind,
    visible: bool,
    timer: Option<Task<()>>,
}

impl Pet {
    pub fn new(cx: &mut Context<Self>, catalog: PetCatalog, kind: PetKind, visible: bool) -> Self {
        let mut pet = Self {
            animator: None,
            catalog,
            kind,
            visible,
            timer: None,
        };
        pet.select(kind, cx);
        pet
    }

    fn select(&mut self, kind: PetKind, cx: &mut Context<Self>) {
        match Package::from_json(kind.manifest()) {
            Ok(package) => {
                let mut animator =
                    Animator::new(package, !islandvolt_platform_windows::animations_enabled());
                animator.set_visible(self.visible);
                self.kind = kind;
                self.animator = Some(animator);
                self.arm(cx);
                cx.notify();
            }
            Err(error) => eprintln!("Не удалось выбрать питомца: {error}"),
        }
    }

    pub fn name(&self) -> &str {
        self.animator.as_ref().map_or("Питомец", Animator::name)
    }

    pub fn cycle(&mut self, cx: &mut Context<Self>) -> PetKind {
        self.select(self.kind.next(), cx);
        self.react(cx);
        self.kind
    }

    pub fn set_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.visible = visible;
        if self
            .animator
            .as_mut()
            .is_some_and(|animator| animator.set_visible(visible))
        {
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
            .size(px(42.0))
            .flex_shrink_0()
            .cursor_pointer()
            .on_click(cx.listener(|pet, _, _, cx| {
                cx.stop_propagation();
                pet.react(cx);
            }))
            .when_some(
                self.animator
                    .as_ref()
                    .and_then(|animator| self.catalog.frame(&animator.frame().asset)),
                |view, frame| view.child(img(frame).size_full()),
            )
    }
}

impl crate::island::Island {
    pub(crate) fn sync_pet(&self, cx: &mut Context<Self>) {
        let visible = !self.hidden
            && !self.pet_hidden
            && !self.show_sources
            && self.mixer.is_none()
            && !self.show_launcher;
        self.pet.update(cx, |pet, cx| pet.set_visible(visible, cx));
    }
}
