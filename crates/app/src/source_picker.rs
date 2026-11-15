use crate::island::Island;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px, rgb};

pub fn render(island: &Island, cx: &mut Context<Island>) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .px(px(38.0))
        .py(px(18.0))
        .gap(px(8.0))
        .child(
            div()
                .id("sources-back")
                .text_sm()
                .text_color(rgb(0xededf2))
                .child("Назад к управлению")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_sources = false;
                    cx.notify();
                })),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x999ba9))
                .child("Выберите медиаплеер"),
        )
        .child(
            div()
                .id("sources-scroll")
                .max_h(px(170.0))
                .overflow_y_scroll()
                .child(option(
                    None,
                    "Автоматически (Windows)".into(),
                    island.selected_source.is_none(),
                    cx,
                ))
                .children(island.sources.iter().map(|source| {
                    option(
                        Some(source.id),
                        source.name.clone(),
                        island.selected_source == Some(source.id),
                        cx,
                    )
                })),
        )
        .when(island.sources.is_empty(), |view| {
            view.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x999ba9))
                    .child("Запустите воспроизведение в плеере"),
            )
        })
        .into_any_element()
}

fn option(id: Option<u64>, name: String, selected: bool, cx: &mut Context<Island>) -> AnyElement {
    div()
        .id(("source", id.unwrap_or(0)))
        .px(px(12.0))
        .py(px(8.0))
        .rounded_md()
        .bg(if selected {
            rgb(0x30283b)
        } else {
            rgb(0x15151a)
        })
        .text_sm()
        .text_color(rgb(0xededf2))
        .truncate()
        .child(name)
        .on_click(cx.listener(move |this, _, _, cx| {
            match this.media_monitor.select(id) {
                Ok(()) => {
                    this.show_sources = false;
                    this.media_error = None;
                }
                Err(error) => this.media_error = Some(error.into()),
            }
            cx.notify();
        }))
        .into_any_element()
}
