use std::time::Duration;

use chrono::{Local, Timelike};
use gpui::{Context, IntoElement, Render, Task, Window, div, prelude::*, px, rgb, svg};
use islandvolt_platform_windows::media::{self, MediaAction, MediaMonitor, Track};

pub struct Island {
    show_date: bool,
    _clock: Task<()>,
    track: Option<Track>,
    media_error: Option<String>,
    media_monitor: MediaMonitor,
    _media_task: Task<()>,
}

impl Island {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let executor = cx.background_executor().clone();
        let clock = cx.spawn(async move |this, cx| {
            loop {
                let seconds = 60 - Local::now().second();
                executor
                    .timer(Duration::from_secs(u64::from(seconds)))
                    .await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        let (monitor, updates) = media::watch();
        let media_task = cx.spawn(async move |this, cx| {
            while let Ok(result) = updates.recv().await {
                if this
                    .update(cx, |island, cx| {
                        match result {
                            Ok(track) => island.track = track,
                            Err(error) => island.media_error = Some(error),
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            show_date: false,
            _clock: clock,
            track: None,
            media_error: None,
            media_monitor: monitor,
            _media_task: media_task,
        }
    }
}

impl Render for Island {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Local::now();
        let secondary = if let Some(error) = &self.media_error {
            error.clone()
        } else if self.show_date {
            now.format("%d.%m.%Y").to_string()
        } else if let Some(track) = &self.track {
            if track.title.trim().is_empty() {
                "МЕДИА".to_string()
            } else {
                track.title.clone()
            }
        } else {
            "ISLANDVOLT".to_string()
        };

        div()
            .id("island")
            .size_full()
            .flex()
            .items_center()
            .justify_between()
            .px(px(22.0))
            .bg(rgb(0x101116))
            .text_color(rgb(0xf5f6fa))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_date = !this.show_date;
                this.media_error = None;
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .min_w_0()
                    .max_w(px(125.0))
                    .gap(px(9.0))
                    .child(div().size(px(7.0)).rounded_full().bg(rgb(0x9b8cff)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .text_color(rgb(0xe2e3eb))
                                    .child(secondary),
                            )
                            .when_some(
                                self.track.as_ref().filter(|track| {
                                    !self.show_date
                                        && self.media_error.is_none()
                                        && !track.artist.trim().is_empty()
                                }),
                                |column, track| {
                                    column.child(
                                        div()
                                            .truncate()
                                            .text_size(px(10.0))
                                            .text_color(rgb(0x858793))
                                            .child(track.artist.clone()),
                                    )
                                },
                            ),
                    ),
            )
            .child({
                let track = self.track.as_ref();
                let session_id = track.map_or(0, |track| track.session_id);
                div()
                    .flex()
                    .items_center()
                    .child(media_button(
                        "previous",
                        "icons/previous.svg",
                        track.is_some_and(|track| track.can_previous),
                        session_id,
                        MediaAction::Previous,
                        cx,
                    ))
                    .child(media_button(
                        "toggle",
                        if track.is_some_and(|track| track.playing) {
                            "icons/pause.svg"
                        } else {
                            "icons/play.svg"
                        },
                        track.is_some_and(|track| track.can_toggle),
                        session_id,
                        MediaAction::Toggle,
                        cx,
                    ))
                    .child(media_button(
                        "next",
                        "icons/next.svg",
                        track.is_some_and(|track| track.can_next),
                        session_id,
                        MediaAction::Next,
                        cx,
                    ))
            })
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(now.format("%H:%M").to_string()),
            )
    }
}

fn media_button(
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
