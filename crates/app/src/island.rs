use std::time::Duration;

use chrono::{Local, Timelike};
use gpui::{Context, IntoElement, Render, Task, Window, div, prelude::*, px, rgb};
use islandvolt_platform_windows::media::{self, MediaMonitor, Track};

pub struct Island {
    show_date: bool,
    _clock: Task<()>,
    track: Option<Track>,
    _media_monitor: MediaMonitor,
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
                let track = result.unwrap_or_else(|error| {
                    eprintln!("Медиасессия недоступна: {error}");
                    None
                });
                if this
                    .update(cx, |island, cx| {
                        island.track = track;
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
            _media_monitor: monitor,
            _media_task: media_task,
        }
    }
}

impl Render for Island {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Local::now();
        let secondary = if self.show_date {
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
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .min_w_0()
                    .max_w(px(190.0))
                    .gap(px(9.0))
                    .child(div().size(px(7.0)).rounded_full().bg(rgb(0x9b8cff)))
                    .child(
                        div()
                            .truncate()
                            .text_xs()
                            .text_color(rgb(0x9b9ca9))
                            .child(secondary),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(now.format("%H:%M").to_string()),
            )
    }
}
