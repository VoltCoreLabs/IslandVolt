use crate::{compact::media_button, island::Island};
use gpui::{AnyElement, Context, IntoElement, MouseButton, div, prelude::*, px, rgb, svg};
use islandvolt_platform_windows::{media::MediaAction, volume::Action};

pub fn render(island: &Island, cx: &mut Context<Island>) -> AnyElement {
    let track = island.track.as_ref();
    let id = track.map_or(0, |track| track.session_id);
    let title = track
        .map(|track| track.title.as_str())
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("Нет воспроизведения")
        .to_owned();
    let artist = track
        .map(|track| track.artist.as_str())
        .filter(|artist| !artist.trim().is_empty())
        .unwrap_or("Выберите музыку в плеере")
        .to_owned();
    let percent = island.volume.map_or(0, |volume| volume.percent);
    let muted = island.volume.is_some_and(|volume| volume.muted);
    let enabled = island.volume.is_some();
    let label = island.volume_error.clone().unwrap_or_else(|| {
        if enabled {
            format!("{percent}%")
        } else {
            "Нет звука".into()
        }
    });
    let width_label = if island.wide {
        "Меньше"
    } else {
        "Больше"
    };
    div()
        .id("expanded-panel")
        .size_full()
        .flex()
        .flex_col()
        .px(px(32.0))
        .pt(px(15.0))
        .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
            if this.dragging_volume && event.pressed_button == Some(MouseButton::Left) {
                crate::volume_slider::set_volume(this, event.position.x, false, cx);
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                if this.dragging_volume {
                    crate::volume_slider::set_volume(this, event.position.x, true, cx);
                }
                this.dragging_volume = false;
                cx.notify();
            }),
        )
        .on_mouse_up_out(
            MouseButton::Left,
            cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                if this.dragging_volume {
                    crate::volume_slider::set_volume(this, event.position.x, true, cx);
                    this.expanded = false;
                    this.transition.set(false);
                }
                this.dragging_volume = false;
                cx.notify();
            }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .max_w(px(240.0))
                        .child(
                            div()
                                .truncate()
                                .text_sm()
                                .text_color(rgb(0xf5f6fa))
                                .child(title),
                        )
                        .child(
                            div()
                                .truncate()
                                .text_xs()
                                .text_color(rgb(0x999ba9))
                                .child(artist),
                        ),
                )
                .child(
                    div()
                        .id("size-preset")
                        .px(px(10.0))
                        .py(px(5.0))
                        .rounded_full()
                        .bg(rgb(0x282a35))
                        .text_xs()
                        .text_color(rgb(0xc8c9d4))
                        .child(width_label)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.wide = !this.wide;
                            this.settings.save_width(this.wide);
                            let scale = window.scale_factor();
                            let width = if this.wide { 540.0 } else { 460.0 };
                            if let Err(error) = islandvolt_platform_windows::resize_island(
                                window,
                                (width * scale) as i32,
                                (220.0 * scale) as i32,
                            ) {
                                eprintln!("{error}");
                            }
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(14.0))
                .mt(px(7.0))
                .child(media_button(
                    "panel-previous",
                    "icons/previous.svg",
                    track.is_some_and(|t| t.can_previous),
                    id,
                    MediaAction::Previous,
                    cx,
                ))
                .child(media_button(
                    "panel-toggle",
                    if track.is_some_and(|t| t.playing) {
                        "icons/pause.svg"
                    } else {
                        "icons/play.svg"
                    },
                    track.is_some_and(|t| t.can_toggle),
                    id,
                    MediaAction::Toggle,
                    cx,
                ))
                .child(media_button(
                    "panel-next",
                    "icons/next.svg",
                    track.is_some_and(|t| t.can_next),
                    id,
                    MediaAction::Next,
                    cx,
                )),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .mt(px(10.0))
                .child(
                    div()
                        .id("mute-panel")
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.volume.is_some() {
                                this.volume_error = this
                                    .volume_monitor
                                    .control(Action::ToggleMute)
                                    .err()
                                    .map(str::to_string);
                                cx.notify();
                            }
                        }))
                        .child(
                            svg()
                                .path(if muted {
                                    "icons/muted.svg"
                                } else {
                                    "icons/speaker.svg"
                                })
                                .size(px(16.0))
                                .text_color(rgb(0xc7c8d4)),
                        )
                        .child(div().text_xs().text_color(rgb(0xb4b6c2)).child("Громкость")),
                )
                .child(
                    div()
                        .max_w(px(150.0))
                        .truncate()
                        .text_xs()
                        .text_color(rgb(0xe2e3eb))
                        .child(label),
                ),
        )
        .child(crate::volume_slider::render(percent, enabled, cx))
        .into_any_element()
}
