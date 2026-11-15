use crate::island::Island;
use gpui::{Context, IntoElement, div, prelude::*, px, rgb};

pub fn render(island: &Island, cx: &mut Context<Island>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(5.0))
        .mt(px(5.0))
        .child(
            div()
                .flex()
                .gap(px(18.0))
                .text_xs()
                .text_color(rgb(0xc8c9d4))
                .child(
                    div()
                        .id("pin-panel")
                        .cursor_pointer()
                        .child(if island.pinned {
                            "Открепить панель"
                        } else {
                            "Закрепить панель"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.pinned = !this.pinned;
                            this.settings.save_pinned(this.pinned);
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .id("toggle-pet")
                        .cursor_pointer()
                        .child(if island.pet_hidden {
                            "Показать кота"
                        } else {
                            "Скрыть кота"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.pet_hidden = !this.pet_hidden;
                            this.settings.save_pet_hidden(this.pet_hidden);
                            this.sync_pet(cx);
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(0x858793))
                .child("← → звук · M mute · P кот · Esc свернуть"),
        )
        .child(div().text_size(px(10.0)).text_color(rgb(0x999ba9)).child(
            if island.keyboard.unavailable {
                "Ctrl+Alt+Space недоступна · используйте наведение или трей"
            } else {
                "Ctrl+Alt+Space · открыть / свернуть"
            },
        ))
}
