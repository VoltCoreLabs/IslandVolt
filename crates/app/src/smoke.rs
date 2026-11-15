//! Runtime layout regression check, enabled only by --smoke-test.
use crate::island::Island;
use gpui::{App, WindowHandle};
use std::time::Duration;

pub fn check_layout(handle: WindowHandle<Island>, cx: &mut App) {
    let executor = cx.background_executor().clone();
    cx.spawn(async move |cx| {
        for (wide, expanded) in [(false, false), (false, true), (true, true), (true, false)] {
            executor.timer(Duration::from_millis(350)).await;
            if let Err(error) = handle.update(cx, |island, _, cx| {
                island.hover.pending.take();
                island.pinned = true;
                island.wide = wide;
                island.expanded = expanded;
                island.transition.set(expanded);
                cx.notify();
            }) {
                eprintln!("Smoke: не удалось изменить состояние окна: {error}");
                return;
            }
            executor.timer(Duration::from_millis(500)).await;
            let result = handle.update(cx, |island, window, _| {
                let actual = window.viewport_size();
                let expected = crate::layout::dimensions(wide, f32::from(expanded));
                if (f32::from(actual.width) - expected.0).abs() > 2.0
                    || (f32::from(actual.height) - expected.1).abs() > 2.0
                {
                    eprintln!("Smoke: GPUI viewport {actual:?} не соответствует {expected:?}; native {:?}, transition {}", window.bounds(), island.transition.value());
                }
            });
            if let Err(error) = result {
                eprintln!("Smoke: не удалось проверить viewport: {error}");
                return;
            }
        }
    })
    .detach();
}
