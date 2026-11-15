use crate::island::Island;
use gpui::{Context, Task, Window};
use islandvolt_platform_windows::tray::{Tray, TrayAction};

pub fn create(window: &Window, cx: &mut Context<Island>) -> (Option<Tray>, Option<Task<()>>) {
    match Tray::new() {
        Ok((tray, events)) => {
            let task = cx.spawn_in(window, async move |this, cx| {
                while let Ok(action) = events.recv().await {
                    if this
                        .update_in(cx, |island, window, cx| match action {
                            TrayAction::Quit => cx.quit(),
                            TrayAction::Show | TrayAction::Hide => {
                                let visible = matches!(action, TrayAction::Show);
                                if !visible {
                                    island.release_keyboard(window);
                                    island.show_sources = false;
                                    island.dragging_volume = false;
                                }
                                if let Err(error) =
                                    islandvolt_platform_windows::set_visible(window, visible)
                                {
                                    eprintln!("Не удалось изменить видимость острова: {error}");
                                } else {
                                    island.hidden = !visible;
                                    island.sync_pet(cx);
                                    if !visible {
                                        island.expanded = false;
                                        island.transition.set(false);
                                    } else if island.pinned {
                                        island.expanded = true;
                                        island.transition.set(true);
                                    }
                                    cx.notify();
                                }
                            }
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            });
            (Some(tray), Some(task))
        }
        Err(error) => {
            eprintln!("Не удалось создать трей IslandVolt: {error}");
            (None, None)
        }
    }
}
