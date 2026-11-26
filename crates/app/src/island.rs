use std::time::Duration;

use crate::transition::Transition;
use chrono::{Local, Timelike};
use gpui::{Context, IntoElement, Render, Task, Window, div, prelude::*, px, size};
use islandvolt_platform_windows::media::{self, MediaMonitor, Source, Track};
use islandvolt_platform_windows::tray::Tray;
use islandvolt_platform_windows::volume::{self, VolumeMonitor, VolumeState};

pub struct Island {
    pub(crate) pet: gpui::Entity<crate::pet::Pet>,
    pub(crate) pet_hidden: bool,
    pub(crate) sources: Vec<Source>,
    pub(crate) selected_source: Option<u64>,
    pub(crate) show_sources: bool,
    pub(crate) mixer: Option<crate::mixer_panel::MixerPanel>,
    pub(crate) hover: crate::hover::Hover,
    pub(crate) hidden: bool,
    pub(crate) keyboard: crate::keyboard::Keyboard,
    _tray: Option<Tray>,
    _tray_task: Option<Task<()>>,
    pub(crate) pinned: bool,
    pub(crate) focus: gpui::FocusHandle,
    pub(crate) settings: crate::settings::Settings,
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
    pub(crate) track_artwork: Option<std::sync::Arc<gpui::RenderImage>>,
    pub(crate) media_error: Option<String>,
    pub(crate) media_monitor: MediaMonitor,
    _media_task: Task<()>,
}

impl Island {
    pub fn new(
        window: &Window,
        cx: &mut Context<Self>,
        settings: crate::settings::Settings,
        preferences: crate::settings::Preferences,
        pet_catalog: crate::pet_catalog::PetCatalog,
    ) -> Self {
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
        let media_task = cx.spawn_in(window, async move |this, cx| {
            while let Ok(result) = updates.recv().await {
                if this
                    .update_in(cx, |island, window, cx| {
                        match result {
                            Ok(state) => {
                                let cover = state
                                    .track
                                    .as_ref()
                                    .and_then(|track| track.artwork.as_ref());
                                let previous = island
                                    .track
                                    .as_ref()
                                    .and_then(|track| track.artwork.as_ref());
                                if previous != cover {
                                    if let Some(outdated) = island.track_artwork.take() {
                                        let _ = window.drop_image(outdated);
                                    }
                                    island.track_artwork =
                                        cover.and_then(|art| crate::track_artwork::prepare(art));
                                }
                                island.track = state.track;
                                island.sources = state.sources;
                                island.selected_source = state.selected;
                            }
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
        let (tray, tray_task) = crate::tray::create(window, cx);
        let keyboard = crate::keyboard::Keyboard::new(window, cx);
        let mut transition = Transition::new(islandvolt_platform_windows::animations_enabled());
        transition.set(preferences.pinned);
        Self {
            pet: cx.new(|cx| {
                crate::pet::Pet::new(
                    cx,
                    pet_catalog,
                    preferences.pet_kind,
                    !preferences.pet_hidden,
                )
            }),
            pet_hidden: preferences.pet_hidden,
            sources: Vec::new(),
            selected_source: None,
            show_sources: false,
            mixer: None,
            hover: crate::hover::Hover {
                inside: false,
                pending: None,
            },
            hidden: false,
            keyboard,
            _tray: tray,
            _tray_task: tray_task,
            pinned: preferences.pinned,
            focus: cx.focus_handle(),
            settings,
            expanded: preferences.pinned,
            wide: preferences.wide,
            dragging_volume: false,
            last_volume_command: None,
            transition,
            last_size: None,
            volume: None,
            volume_error: None,
            volume_monitor,
            _volume_task: volume_task,
            _clock: clock,
            track: None,
            track_artwork: None,
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
        let (width, height) = crate::layout::dimensions(self.wide, progress);
        let scale = window.scale_factor();
        let physical = (
            (width * scale).round() as i32,
            (height * scale).round() as i32,
        );
        if self.last_size != Some(physical) {
            let previous_width = self.last_size.map(|(width, _)| width);
            self.last_size = Some(physical);
            // GPUI queues the native resize outside the borrowed Window update.
            // A direct SetWindowPos here reenters WM_SIZE before GPUI can update its viewport.
            window.resize(size(px(width), px(height)));
            if previous_width != Some(physical.0) {
                window.defer(cx, move |window, _| {
                    if let Err(error) =
                        islandvolt_platform_windows::position_island(window, physical.0)
                    {
                        eprintln!("Не удалось расположить остров у верхнего края: {error}");
                    }
                });
            }
        }
        if active && !self.hidden {
            window.request_animation_frame();
        }
        let content = if progress >= 0.15 || (self.expanded && !active) {
            div()
                .w_full()
                .h(px(crate::layout::EXPANDED_HEIGHT))
                .opacity(((progress - 0.15) / 0.35).clamp(0.0, 1.0))
                .child(crate::panel::render(self, cx))
                .into_any_element()
        } else {
            self.compact(cx)
        };
        div()
            .id("island-shell")
            .w(px(width))
            .h(px(height))
            .relative()
            .overflow_hidden()
            .font_family("Golos Text")
            .track_focus(&self.focus)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, _| window.focus(&this.focus)),
            )
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.key_down(event, window, cx);
            }))
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.hidden {
                    return;
                }
                this.hover_changed(*hovered, cx);
            }))
            .child(crate::silhouette::background(width, height))
            .child(content)
    }
}
