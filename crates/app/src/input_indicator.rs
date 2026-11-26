use crate::{island::Island, theme::Palette};
use gpui::{Context, IntoElement, Task, div, prelude::*, px};
use islandvolt_platform_windows::input_language::{self, InputLanguage, LanguageMonitor};
use std::time::Duration;

pub(crate) struct InputIndicator {
    language: Option<InputLanguage>,
    pub(crate) received: bool,
    highlighted: bool,
    _monitor: LanguageMonitor,
    _updates: Task<()>,
    clear_highlight: Option<Task<()>>,
}

impl InputIndicator {
    pub fn new(cx: &mut Context<Island>) -> Self {
        let (monitor, updates) = input_language::watch();
        let task = cx.spawn(async move |this, cx| {
            while let Ok(update) = updates.recv().await {
                if this
                    .update(cx, |island, cx| {
                        let indicator = &mut island.input_indicator;
                        let language = update.unwrap_or_default();
                        let changed = indicator.received
                            && language.is_some()
                            && indicator.language != language;
                        indicator.language = language;
                        indicator.received = true;
                        indicator.clear_highlight.take();
                        indicator.highlighted = changed;
                        if changed {
                            let timer = cx.background_executor().timer(Duration::from_millis(1800));
                            indicator.clear_highlight = Some(cx.spawn(async move |this, cx| {
                                timer.await;
                                let _ = this.update(cx, |island, cx| {
                                    island.input_indicator.highlighted = false;
                                    cx.notify();
                                });
                            }));
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
            language: None,
            received: false,
            highlighted: false,
            _monitor: monitor,
            _updates: task,
            clear_highlight: None,
        }
    }

    pub fn render(&self, palette: Palette) -> impl IntoElement {
        div()
            .id("input-language")
            .min_w(px(32.0))
            .px(px(6.0))
            .py(px(3.0))
            .rounded_full()
            .bg(if self.highlighted {
                palette.selected
            } else {
                palette.surface
            })
            .text_size(px(11.0))
            .text_color(if self.highlighted {
                palette.accent
            } else {
                palette.secondary
            })
            .child(
                self.language
                    .as_ref()
                    .map_or("—", |language| language.label.as_str())
                    .to_owned(),
            )
    }
}
