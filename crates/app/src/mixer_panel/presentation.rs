use super::MixerPanel;
use crate::theme::Palette;
use crate::{island::Island, volume_slider};
use gpui::{AnyElement, Context, IntoElement, MouseButton, div, prelude::*, px, svg};
use islandvolt_platform_windows::mixer::{Action, Session};
use std::time::Instant;

impl MixerPanel {
    pub fn render(&self, palette: Palette, cx: &mut Context<Island>) -> AnyElement {
        let output = self
            .snapshot
            .as_ref()
            .and_then(|state| state.output.as_deref())
            .unwrap_or("Микшер приложений");
        let warning = self.command_error.or_else(|| {
            self.snapshot
                .as_ref()
                .and_then(|state| state.warning.as_deref())
        });
        let empty = if self.snapshot.is_none() {
            "Загружаем аудиосессии…"
        } else {
            "Запустите звук в нужном приложении"
        };
        div()
            .id("mixer-panel")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(14.0))
            .gap(px(6.0))
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if event.pressed_button == Some(MouseButton::Left) {
                    this.move_mixer_slider(event.position.x, false, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.move_mixer_slider(event.position.x, true, cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.move_mixer_slider(event.position.x, true, cx);
                }),
            )
            .child(
                div()
                    .w(px(volume_slider::WIDTH))
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_xs()
                    .text_color(palette.secondary)
                    .child(
                        div()
                            .id("mixer-back")
                            .cursor_pointer()
                            .child(
                                svg()
                                    .path("icons/back.svg")
                                    .size(px(15.0))
                                    .text_color(palette.text),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.close_mixer(cx))),
                    )
                    .child(div().truncate().max_w(px(188.0)).child(output.to_owned()))
                    .child(
                        div()
                            .id("mixer-refresh")
                            .cursor_pointer()
                            .child("Обновить")
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(panel) = &mut this.mixer {
                                    panel.command_error = panel.monitor.refresh().err();
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .when(self.rows().is_empty() && warning.is_none(), |view| {
                view.child(
                    div()
                        .pt(px(28.0))
                        .text_sm()
                        .text_color(palette.secondary)
                        .child(empty),
                )
            })
            .when(!self.rows().is_empty(), |view| {
                view.child(
                    div()
                        .id("mixer-scroll")
                        .track_scroll(&self.scroll)
                        .w(px(volume_slider::WIDTH))
                        .max_h(px(167.0))
                        .overflow_y_scroll()
                        .children(self.rows().iter().map(|row| self.row(row, palette, cx))),
                )
            })
            .when_some(warning.map(str::to_owned), |view, warning| {
                view.child(
                    div()
                        .w(px(volume_slider::WIDTH))
                        .text_xs()
                        .truncate()
                        .text_color(palette.warning)
                        .child(warning),
                )
            })
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(palette.secondary)
                    .child("↑ ↓ сессия · ← → громкость · M звук · Esc назад"),
            )
            .into_any_element()
    }

    fn row(&self, row: &Session, palette: Palette, cx: &mut Context<Island>) -> AnyElement {
        let id = row.id;
        let muted = row.volume.muted;
        div()
            .id(("mixer-session", id))
            .pb(px(6.0))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .text_xs()
                    .text_color(if self.selection == Some(id) {
                        palette.accent
                    } else {
                        palette.text
                    })
                    .child(
                        div()
                            .id(("session-mute", id))
                            .cursor_pointer()
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
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(panel) = &mut this.mixer {
                                    panel.selection = Some(id);
                                    panel.queue(id, Action::ToggleMute);
                                    cx.notify();
                                }
                            })),
                    )
                    .child(div().flex_1().min_w_0().truncate().child(row.name.clone()))
                    .child(div().child(format!("{}%", row.volume.percent))),
            )
            .child(
                div()
                    .id(("session-level", id))
                    .w_full()
                    .h(px(22.0))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                            if let Some(panel) = &mut this.mixer {
                                panel.selection = Some(id);
                                panel.drag = Some((id, Instant::now()));
                                panel.queue(
                                    id,
                                    Action::Set(volume_slider::slider_percent(
                                        f32::from(event.position.x),
                                        this.wide,
                                    )),
                                );
                                cx.notify();
                            }
                        }),
                    )
                    .child(volume_slider::track(row.volume.percent, palette)),
            )
            .into_any_element()
    }
}
