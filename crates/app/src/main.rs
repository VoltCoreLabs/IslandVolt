mod assets;
mod compact;
mod hover;
mod input_indicator;
mod island;
mod keyboard;
mod launcher_panel;
mod layout;
mod mixer_panel;
mod panel;
mod panel_footer;
mod panel_header;
mod pet;
mod pet_catalog;
mod settings;
mod silhouette;
mod smoke;
mod source_picker;
mod theme;
mod track_artwork;
mod transition;
mod tray;
mod volume_slider;

use gpui::{
    App, AppContext, Application, Bounds, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, point, px, size,
};
use island::Island;

fn main() {
    let smoke_test = std::env::args().any(|argument| argument == "--smoke-test");
    let (settings, preferences, shutdown) = settings::Settings::open();
    let pet_catalog = pet_catalog::PetCatalog::load();
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
            let size = size(
                px(crate::layout::dimensions(preferences.wide, 0.0).0),
                px(crate::layout::dimensions(preferences.wide, 0.0).1),
            );
            let mut bounds = Bounds::centered(None, size, cx);
            if let Some(display) = cx.primary_display() {
                bounds.origin = point(bounds.origin.x, display.bounds().origin.y);
            }
            let opened = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: None,
                    kind: WindowKind::PopUp,
                    window_background: WindowBackgroundAppearance::Transparent,
                    is_resizable: false,
                    focus: false,
                    ..Default::default()
                },
                move |window, cx| {
                    if let Err(error) = islandvolt_platform_windows::configure_island(window) {
                        eprintln!("Не удалось настроить окно острова: {error}");
                    }
                    window.on_window_should_close(cx, |_, cx| {
                        cx.quit();
                        true
                    });
                    cx.new(|cx| Island::new(window, cx, settings, preferences, pet_catalog))
                },
            );
            match opened {
                Ok(handle) if smoke_test => smoke::check_layout(handle, cx),
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Не удалось открыть окно IslandVolt: {error}");
                    cx.quit();
                }
            }
            if smoke_test {
                let timer = cx
                    .background_executor()
                    .timer(std::time::Duration::from_secs(7));
                cx.spawn(async move |cx| {
                    timer.await;
                    let _ = cx.update(|cx| cx.quit());
                })
                .detach();
            }
        });
    shutdown.finish();
}
