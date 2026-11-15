use std::time::Duration;

use crate::transition::Transition;
use chrono::{Local, Timelike};
use gpui::{Context, IntoElement, Render, Task, Window, div, prelude::*, rgb};
use islandvolt_platform_windows::media::{self, MediaMonitor, Track};
use islandvolt_platform_windows::volume::{self, VolumeMonitor, VolumeState};

pub struct Island {
    pub(crate) expanded: bool,
    pub(crate) wide: bool,
    pub(crate) dragging_volume: bool,
    pub(crate) last_volume_command: Option<std::time::Instant>,
    pub(crate) transition: Transition,
    last_size: Option<(i32, i32)>,
    pub(crate) volume: Option<VolumeState>,
    pub(crate) volume_error: Option<String>,
    pub(crate) volume_monitor: VolumeMonitor,
    _volume_task: Task<()>,
    _clock: Task<()>,
    pub(crate) track: Option<Track>,
    pub(crate) media_error: Option<String>,
    pub(crate) media_monitor: MediaMonitor,
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
        let (volume_monitor, volume_updates) = volume::watch();
        let volume_task = cx.spawn(async move |this, cx| {
            while let Ok(result) = volume_updates.recv().await {
                if this
                    .update(cx, |island, cx| {
                        match result {
                            Ok(volume) => {
                                island.volume = Some(volume);
                                island.volume_error = None;
                            }
                            Err(error) => {
                                island.volume = None;
                                island.volume_error = Some(error);
                            }
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
            expanded: false,
            wide: false,
            dragging_volume: false,
            last_volume_command: None,
            transition: Transition::new(islandvolt_platform_windows::animations_enabled()),
            last_size: None,
            volume: None,
            volume_error: None,
            volume_monitor,
            _volume_task: volume_task,
            _clock: clock,
            track: None,
            media_error: None,
            media_monitor: monitor,
            _media_task: media_task,
        }
    }
}

impl Render for Island {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let progress = self.transition.value();
        let active = self.transition.active();
        let expanded_width = if self.wide { 540.0 } else { 460.0 };
        let width = 400.0 + (expanded_width - 400.0) * progress;
        let height = 76.0 + 144.0 * progress;
        let radius = 24.0;
        let scale = window.scale_factor();
        let physical = (
            (width * scale).round() as i32,
            (height * scale).round() as i32,
        );
        if self.last_size != Some(physical) {
            self.last_size = Some(physical);
            window.defer(cx, move |window, _| {
                if let Err(error) = islandvolt_platform_windows::resize_island(
                    window,
                    physical.0,
                    physical.1,
                    (radius * scale).round() as i32,
                ) {
                    eprintln!("Не удалось изменить размер острова: {error}");
                }
            });
        }
        if active {
            window.request_animation_frame();
        }
        let content = if self.expanded {
            crate::panel::render(self, cx)
        } else {
            self.compact(cx)
        };
        div()
            .id("island-shell")
            .size_full()
            .bg(rgb(0x08080b))
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if *hovered || !this.dragging_volume {
                    this.expanded = *hovered;
                    this.transition.set(*hovered);
                    cx.notify();
                }
            }))
            .child(content)
    }
}
