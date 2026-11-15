mod assets;
mod compact;
mod island;
mod panel;
mod transition;
mod volume_slider;

use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, point, px, size};
use island::Island;

fn main() {
    Application::new()
        .with_assets(assets::Icons)
        .run(|cx: &mut App| {
            let size = size(px(400.0), px(76.0));
            let mut bounds = Bounds::centered(None, size, cx);
            if let Some(display) = cx.primary_display() {
                bounds.origin = point(bounds.origin.x, display.bounds().origin.y);
            }
            if let Err(error) = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: None,
                    is_resizable: false,
                    focus: false,
                    ..Default::default()
                },
                |window, cx| {
                    if let Err(error) = islandvolt_platform_windows::configure_island(window) {
                        eprintln!("Не удалось настроить окно острова: {error}");
                    }
                    cx.new(Island::new)
                },
            ) {
                eprintln!("Не удалось открыть окно IslandVolt: {error}");
                cx.quit();
            }
        });
}
