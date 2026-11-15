mod assets;
mod compact;
mod island;
mod panel;
mod settings;
mod transition;
mod volume_slider;

use gpui::{
    App, AppContext, Application, Bounds, WindowBackgroundAppearance, WindowBounds, WindowOptions,
    point, px, size,
};
use island::Island;

fn main() {
    let (settings, wide) = settings::Settings::open();
    Application::new()
        .with_assets(assets::Icons)
        .run(move |cx: &mut App| {
            if let Err(error) =
                cx.text_system()
                    .add_fonts(vec![std::borrow::Cow::Borrowed(include_bytes!(
                        "../../../assets/fonts/golos/GolosText.ttf"
                    ))])
            {
                eprintln!("Не удалось загрузить Golos Text: {error}");
            }
            let size = size(px(400.0), px(76.0));
            let mut bounds = Bounds::centered(None, size, cx);
            if let Some(display) = cx.primary_display() {
                bounds.origin = point(bounds.origin.x, display.bounds().origin.y);
            }
            if let Err(error) = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Opaque,
                    is_resizable: false,
                    focus: false,
                    ..Default::default()
                },
                move |window, cx| {
                    if let Err(error) = islandvolt_platform_windows::configure_island(window) {
                        eprintln!("Не удалось настроить окно острова: {error}");
                    }
                    cx.new(|cx| Island::new(cx, settings, wide))
                },
            ) {
                eprintln!("Не удалось открыть окно IslandVolt: {error}");
                cx.quit();
            }
        });
}
