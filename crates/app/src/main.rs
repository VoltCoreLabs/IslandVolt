use gpui::{
    App, Application, Bounds, Context, IntoElement, Render, Window, WindowBounds, WindowOptions,
    div, point, prelude::*, px, rgb, size,
};

struct Island;

impl Render for Island {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x101116))
            .text_color(rgb(0xffffff))
            .child("IslandVolt")
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let size = size(px(360.0), px(56.0));
        let mut bounds = Bounds::centered(None, size, cx);
        if let Some(display) = cx.primary_display() {
            bounds.origin = point(bounds.origin.x, display.bounds().origin.y + px(12.0));
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
                cx.new(|_| Island)
            },
        ) {
            eprintln!("Не удалось открыть окно IslandVolt: {error}");
            cx.quit();
        }
    });
}
