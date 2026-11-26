use crate::island::Island;
use crate::theme::Palette;
use chrono::Local;
use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use islandvolt_platform_windows::media::MediaAction;

impl Island {
    pub(crate) fn compact(&self, palette: Palette) -> gpui::AnyElement {
        let title = self
            .media_error
            .clone()
            .or_else(|| {
                self.track
                    .as_ref()
                    .map(|track| track.title.clone())
                    .filter(|title| !title.trim().is_empty())
            })
            .unwrap_or_else(|| "IslandVolt".into());
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_between()
            .px(px(38.0))
            .pb(px(7.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .min_w_0()
                    .max_w(px(230.0))
                    .when(!self.pet_hidden, |view| view.child(self.pet.clone()))
                    .when(self.track_artwork.is_some(), |view| {
                        view.child(crate::track_artwork::thumbnail(
                            self.track_artwork.as_ref(),
                            28.0,
                            palette,
                        ))
                    })
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .text_color(palette.text)
                            .child(title),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.secondary)
                    .child(Local::now().format("%H:%M").to_string()),
            )
            .into_any_element()
    }
}

pub(crate) fn media_button(
    id: &'static str,
    icon: &'static str,
    enabled: bool,
    session_id: u64,
    action: MediaAction,
    palette: Palette,
    cx: &mut Context<Island>,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(26.0))
        .rounded_full()
        .text_color(if enabled {
            palette.text
        } else {
            palette.disabled
        })
        .on_click(cx.listener(move |this, _, _, cx| {
            cx.stop_propagation();
            if enabled {
                this.media_error = this
                    .media_monitor
                    .control(session_id, action)
                    .err()
                    .map(str::to_string);
                cx.notify();
            }
        }))
        .child(svg().path(icon).size(px(14.0)).text_color(if enabled {
            palette.text
        } else {
            palette.disabled
        }))
}
