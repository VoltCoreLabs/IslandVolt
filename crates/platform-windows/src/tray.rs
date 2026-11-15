//! Windows tray icon and event-driven menu commands. Owned by the UI thread.
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

#[derive(Clone, Copy, Debug)]
pub enum TrayAction {
    Show,
    Hide,
    Quit,
}

pub struct Tray {
    _icon: TrayIcon,
}

impl Tray {
    /// Create on the thread running the Windows message loop.
    pub fn new() -> anyhow::Result<(Self, async_channel::Receiver<TrayAction>)> {
        let menu = Menu::new();
        let show = MenuItem::new("Показать остров", true, None);
        let hide = MenuItem::new("Скрыть остров", true, None);
        let quit = MenuItem::new("Завершить IslandVolt", true, None);
        menu.append_items(&[&show, &hide, &quit])?;
        let (sender, receiver) = async_channel::bounded(8);
        let show_id = show.id().clone();
        let hide_id = hide.id().clone();
        let quit_id = quit.id().clone();
        let icon = TrayIconBuilder::new()
            .with_tooltip("IslandVolt")
            .with_menu(Box::new(menu))
            .with_icon(icon()?)
            .build()?;
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == show_id {
                TrayAction::Show
            } else if event.id == hide_id {
                TrayAction::Hide
            } else if event.id == quit_id {
                TrayAction::Quit
            } else {
                return;
            };
            let _ = sender.try_send(action);
        }));
        Ok((Self { _icon: icon }, receiver))
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
    }
}

fn icon() -> anyhow::Result<Icon> {
    let svg = include_bytes!("../../../assets/icons/phosphor/waveform.svg");
    let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default())?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(32, 32)
        .ok_or_else(|| anyhow::anyhow!("Не удалось создать иконку трея"))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(
            32.0 / tree.size().width(),
            32.0 / tree.size().height(),
        ),
        &mut pixmap.as_mut(),
    );
    // Phosphor's SVG alpha becomes a bright monochrome icon for dark/light trays.
    let rgba = pixmap
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|pixel| [190, 160, 255, pixel[3]])
        .collect();
    Ok(Icon::from_rgba(rgba, 32, 32)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn phosphor_tray_icon_can_be_rasterized() {
        assert!(super::icon().is_ok());
    }
}
