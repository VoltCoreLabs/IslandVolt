use crate::island::Island;
use crate::theme::Palette;
use gpui::{Context, IntoElement, MouseButton, Pixels, div, prelude::*, px};
use islandvolt_platform_windows::volume::Action;
use std::time::{Duration, Instant};

pub(crate) const WIDTH: f32 = 300.0;

pub fn set_volume(island: &mut Island, x: Pixels, final_value: bool, cx: &mut Context<Island>) {
    if island.volume.is_none() {
        return;
    }
    let now = Instant::now();
    if !final_value
        && island
            .last_volume_command
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(50))
    {
        return;
    }
    island.last_volume_command = Some(now);
    island.volume_error = island
        .volume_monitor
        .control(Action::Set(slider_percent(f32::from(x), island.wide)))
        .err()
        .map(str::to_string);
    cx.notify();
}

pub(crate) fn slider_percent(x: f32, wide: bool) -> u8 {
    let left = (crate::layout::dimensions(wide, 1.0).0 - WIDTH) / 2.0;
    (((x - left) / WIDTH).clamp(0.0, 1.0) * 100.0).round() as u8
}

pub fn render(
    percent: u8,
    enabled: bool,
    palette: Palette,
    cx: &mut Context<Island>,
) -> impl IntoElement {
    div()
        .id("volume-slider")
        .w(px(WIDTH))
        .h(px(24.0))
        .flex()
        .items_center()
        .when(enabled, |slider| {
            slider.cursor_pointer().on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                    this.dragging_volume = true;
                    set_volume(this, event.position.x, true, cx);
                }),
            )
        })
        .child(track(percent, palette))
}

pub(crate) fn track(percent: u8, palette: Palette) -> impl IntoElement {
    div()
        .relative()
        .w_full()
        .h(px(6.0))
        .rounded_full()
        .bg(palette.slider_track)
        .child(
            div()
                .w(px(WIDTH * f32::from(percent) / 100.0))
                .h_full()
                .rounded_full()
                .bg(palette.accent),
        )
        .child(
            div()
                .absolute()
                .left(px(
                    (WIDTH * f32::from(percent) / 100.0 - 6.0).clamp(0.0, WIDTH - 12.0)
                ))
                .top(px(-3.0))
                .size(px(12.0))
                .rounded_full()
                .bg(palette.slider_thumb),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slider_clamps_and_maps_position() {
        for wide in [false, true] {
            let center = crate::layout::dimensions(wide, 1.0).0 / 2.0;
            assert_eq!(slider_percent(0.0, wide), 0);
            assert_eq!(slider_percent(center, wide), 50);
            assert_eq!(slider_percent(999.0, wide), 100);
        }
    }
}
