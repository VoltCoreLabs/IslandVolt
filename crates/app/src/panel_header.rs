use crate::{island::Island, theme::Palette};
use gpui::{Context, IntoElement, div, prelude::*, px};

pub fn render(island: &Island, palette: Palette, cx: &mut Context<Island>) -> impl IntoElement {
    let track = island.track.as_ref();
    let title = island.media_error.clone().unwrap_or_else(|| {
        track
            .map(|track| track.title.as_str())
            .filter(|title| !title.trim().is_empty())
            .unwrap_or("Нет воспроизведения")
            .to_owned()
    });
    let artist = track
        .map(|track| track.artist.as_str())
        .filter(|artist| !artist.trim().is_empty())
        .unwrap_or("Выберите музыку в плеере")
        .to_owned();
    let width_label = if island.wide {
        "Меньше"
    } else {
        "Больше"
    };
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .child(crate::track_artwork::thumbnail(
            island.track_artwork.as_ref(),
            40.0,
            palette,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .flex_1()
                .child(
                    div()
                        .truncate()
                        .text_sm()
                        .text_color(palette.text)
                        .child(title),
                )
                .child(
                    div()
                        .truncate()
                        .text_xs()
                        .text_color(palette.secondary)
                        .child(artist),
                ),
        )
        .child(
            div()
                .id("size-preset")
                .cursor_pointer()
                .flex_shrink_0()
                .px(px(10.0))
                .py(px(5.0))
                .rounded_full()
                .bg(palette.surface)
                .text_xs()
                .text_color(palette.text)
                .child(width_label)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.wide = !this.wide;
                    this.settings.save_width(this.wide);
                    cx.notify();
                })),
        )
        .child(
            div()
                .id("cycle-theme")
                .cursor_pointer()
                .flex_shrink_0()
                .rounded_full()
                .px(px(8.0))
                .py(px(5.0))
                .bg(palette.surface)
                .text_xs()
                .text_color(palette.text)
                .child(format!("Тема · {}", island.theme.label()))
                .on_click(cx.listener(|this, _, _, cx| this.cycle_theme(cx))),
        )
}
