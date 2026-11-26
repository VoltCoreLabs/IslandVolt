//! Runtime layout regression check, enabled only by --smoke-test.
use crate::island::Island;
use gpui::{App, WindowHandle};
use std::time::Duration;

pub fn check_layout(handle: WindowHandle<Island>, cx: &mut App) {
    let executor = cx.background_executor().clone();
    cx.spawn(async move |cx| {
        for (wide, expanded) in [(false, false), (false, true), (true, true), (true, false)] {
            executor.timer(Duration::from_millis(350)).await;
            let outcome = handle.update(cx, |island, _, cx| {
                island.hover.pending.take();
                island.pinned = true;
                island.wide = wide;
                if expanded {
                    island.hover_changed(true, cx);
                } else {
                    island.expanded = false;
                    island.transition.set(false);
                }
                cx.notify();
                island.transition.active()
            });
            let animated = match outcome {
                Ok(animated) => animated,
                Err(error) => {
                    eprintln!("Smoke: не удалось изменить состояние окна: {error}");
                    return;
                }
            };
            if expanded && animated {
                let mut saw_intermediate_height = false;
                for _ in 0..8 {
                    executor.timer(Duration::from_millis(25)).await;
                    if let Ok(height) = handle.update(cx, |_, window, _| f32::from(window.viewport_size().height)) {
                        let compact = crate::layout::dimensions(wide, 0.0).1;
                        saw_intermediate_height |= height > compact + 2.0
                            && height < crate::layout::EXPANDED_HEIGHT - 2.0;
                    }
                }
                if saw_intermediate_height {
                    println!("Smoke: зафиксирована промежуточная высота при наведении");
                } else {
                    eprintln!("Smoke: пропущена промежуточная высота при наведении");
                }
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
        if handle.update(cx, |island, _, cx| {
            island.hover_changed(true, cx);
            island.open_mixer(cx);
        }).is_err() {
            eprintln!("Smoke: не удалось открыть микшер");
            return;
        }
        executor.timer(Duration::from_millis(700)).await;
        match handle.update(cx, |island, _, cx| {
            let ready = island.mixer.as_ref().is_some_and(|mixer| mixer.ready());
            let handled = island.mixer_key("escape", cx);
            ready && handled && island.mixer.is_none()
        }) {
            Ok(true) => println!("Smoke: панель микшера получила снимок и закрылась по Esc"),
            _ => eprintln!("Smoke: микшер не получил снимок или не закрылся"),
        }
    })
    .detach();
}
