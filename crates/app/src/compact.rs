use crate::island::Island;
use chrono::Local;
use gpui::{Context, IntoElement, div, prelude::*, px, rgb, svg};
use islandvolt_platform_windows::media::MediaAction;

impl Island {
    pub(crate) fn compact(&mut self, _cx: &mut Context<Self>) -> gpui::AnyElement {
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
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .min_w_0()
                    .max_w(px(230.0))
                    .child(div().size(px(8.0)).rounded_full().bg(rgb(0xc86d8b)))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .text_color(rgb(0xededf2))
                            .child(title),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xc9c9d2))
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
            rgb(0xf5f6fa)
        } else {
            rgb(0x8a8c9b)
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
            rgb(0xf5f6fa)
        } else {
            rgb(0x8a8c9b)
        }))
}
