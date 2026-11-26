use crate::theme::Palette;
use crate::{compact::media_button, island::Island};
use gpui::{AnyElement, Context, IntoElement, MouseButton, div, prelude::*, px, svg};
use islandvolt_platform_windows::{media::MediaAction, volume::Action};

pub fn render(island: &Island, palette: Palette, cx: &mut Context<Island>) -> AnyElement {
    if island.show_launcher {
        return island.launcher.render(palette, cx);
    }
    if let Some(mixer) = &island.mixer {
        return mixer.render(palette, cx);
    }
    if island.show_sources {
        return crate::source_picker::render(island, palette, cx);
    }
    let track = island.track.as_ref();
    let id = track.map_or(0, |track| track.session_id);
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
    div()
        .id("expanded-panel")
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
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
                    if !this.pinned && !this.keyboard.open {
                        this.expanded = false;
                        this.transition.set(false);
                    }
                }
                this.dragging_volume = false;
                cx.notify();
            }),
        )
        .child(crate::panel_header::render(island, palette, cx))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(14.0))
                .when(!island.pet_hidden, |view| view.child(island.pet.clone()))
                .mt(px(7.0))
                .child(media_button(
                    "panel-previous",
                    "icons/previous.svg",
                    track.is_some_and(|t| t.can_previous),
                    id,
                    MediaAction::Previous,
                    palette,
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
                    palette,
                    cx,
                ))
                .child(media_button(
                    "panel-next",
                    "icons/next.svg",
                    track.is_some_and(|t| t.can_next),
                    id,
                    MediaAction::Next,
                    palette,
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
                                .text_color(palette.secondary),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(palette.secondary)
                                .child("Громкость"),
                        ),
                )
                .child(
                    div()
                        .max_w(px(150.0))
                        .truncate()
                        .text_xs()
                        .text_color(palette.text)
                        .child(label),
                ),
        )
        .child(crate::volume_slider::render(percent, enabled, palette, cx))
        .child(audio_navigation(island, palette, cx))
        .child(crate::panel_footer::render(island, palette, cx))
        .into_any_element()
}

fn audio_navigation(
    island: &Island,
    palette: Palette,
    cx: &mut Context<Island>,
) -> impl IntoElement {
    div()
        .flex()
        .gap(px(18.0))
        .text_xs()
        .text_color(palette.secondary)
        .child(
            div()
                .id("choose-source")
                .cursor_pointer()
                .child(if island.selected_source.is_some() {
                    "Сменить плеер"
                } else {
                    "Плеер: автоматически"
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_sources = true;
                    this.sync_pet(cx);
                    cx.notify();
                })),
        )
        .child(
            div()
                .id("open-mixer")
                .cursor_pointer()
                .child("Микшер приложений")
                .on_click(cx.listener(|this, _, _, cx| this.open_mixer(cx))),
        )
}
