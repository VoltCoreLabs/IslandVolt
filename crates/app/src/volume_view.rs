use crate::island::Island;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px, rgb, svg};
use islandvolt_platform_windows::volume::Action;

pub fn render(island: &Island, cx: &mut Context<Island>) -> AnyElement {
    let available = island.volume.is_some();
    let muted = island.volume.is_some_and(|volume| volume.muted);
    let label = if let Some(error) = &island.volume_error {
        error.clone()
    } else if let Some(volume) = island.volume {
        format!("{}%", volume.percent)
    } else {
        "Звук...".into()
    };
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_between()
        .px(px(20.0))
        .bg(rgb(0x101116))
        .child(
            div()
                .id("back")
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_volume = false;
                    cx.notify();
                }))
                .child(
                    svg()
                        .path("icons/back.svg")
                        .size(px(15.0))
                        .text_color(rgb(0xf5f6fa)),
                ),
        )
        .child(button(
            "mute",
            if muted {
                "icons/muted.svg"
            } else {
                "icons/speaker.svg"
            },
            Action::ToggleMute,
            available,
            cx,
        ))
        .child(button(
            "down",
            "icons/minus.svg",
            Action::Decrease,
            available,
            cx,
        ))
        .child(
            div()
                .min_w(px(50.0))
                .max_w(px(100.0))
                .truncate()
                .text_sm()
                .text_color(rgb(0xf5f6fa))
                .child(label),
        )
        .child(button(
            "up",
            "icons/plus.svg",
            Action::Increase,
            available,
            cx,
        ))
        .into_any_element()
}

fn button(
    id: &'static str,
    icon: &'static str,
    action: Action,
    enabled: bool,
    cx: &mut Context<Island>,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(28.0))
        .on_click(cx.listener(move |this, _, _, cx| {
            if enabled {
                this.volume_error = this
                    .volume_monitor
                    .control(action)
                    .err()
                    .map(str::to_string);
                cx.notify();
            }
        }))
        .child(svg().path(icon).size(px(16.0)).text_color(if enabled {
            rgb(0xf5f6fa)
        } else {
            rgb(0x8a8c9b)
        }))
}
