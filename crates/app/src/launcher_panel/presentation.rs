use super::LauncherPanel;
use crate::{island::Island, theme::Palette};
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px, svg};

impl LauncherPanel {
    pub fn render(&self, palette: Palette, cx: &mut Context<Island>) -> AnyElement {
        div()
            .size_full()
            .px(px(38.0))
            .pt(px(16.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_xs()
                    .text_color(palette.text)
                    .child(
                        div()
                            .id("launcher-back")
                            .cursor_pointer()
                            .child("Назад")
                            .on_click(cx.listener(|island, _, _, cx| {
                                island.show_launcher = false;
                                island.sync_pet(cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("launcher-add")
                            .cursor_pointer()
                            .child(if self.busy {
                                "Подождите…"
                            } else {
                                "Закрепить .exe"
                            })
                            .on_click(cx.listener(|island, _, _, cx| island.add_launcher_app(cx))),
                    ),
            )
            .child(
                div()
                    .id("launcher-scroll")
                    .track_scroll(&self.scroll)
                    .max_h(px(160.0))
                    .overflow_y_scroll()
                    .children(self.paths.iter().enumerate().map(|(index, path)| {
                        let name = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        div()
                            .id(("pinned-app", index))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .py(px(6.0))
                            .bg(if index == self.selection {
                                palette.selected
                            } else {
                                palette.shell
                            })
                            .child(
                                div()
                                    .id(("launch", index))
                                    .flex_1()
                                    .min_w_0()
                                    .cursor_pointer()
                                    .truncate()
                                    .text_sm()
                                    .text_color(palette.text)
                                    .child(name)
                                    .on_click(cx.listener(move |island, _, _, cx| {
                                        island.launch_app(index, cx)
                                    })),
                            )
                            .child(
                                div()
                                    .id(("unpin", index))
                                    .cursor_pointer()
                                    .child(
                                        svg()
                                            .path("icons/minus.svg")
                                            .size(px(16.0))
                                            .text_color(palette.secondary),
                                    )
                                    .on_click(cx.listener(move |island, _, _, cx| {
                                        island.unpin_app(index, cx)
                                    })),
                            )
                            .into_any_element()
                    })),
            )
            .when(self.paths.is_empty(), |view| {
                view.child(
                    div()
                        .text_sm()
                        .text_color(palette.secondary)
                        .child("Закрепите приложение для быстрого запуска"),
                )
            })
            .when_some(self.error.clone(), |view, error| {
                view.child(div().text_xs().text_color(palette.warning).child(error))
            })
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(palette.secondary)
                    .child("A добавить · ↑ ↓ выбор · Enter запуск · Delete убрать · Esc назад"),
            )
            .into_any_element()
    }
}
