mod presentation;
use crate::island::Island;
use gpui::{Context, PathPromptOptions, ScrollHandle, Task};
use std::path::PathBuf;

pub(crate) const MAX_APPS: usize = 12;

pub(crate) struct LauncherPanel {
    paths: Vec<PathBuf>,
    selection: usize,
    pub(crate) busy: bool,
    error: Option<String>,
    task: Option<Task<()>>,
    scroll: ScrollHandle,
}

impl LauncherPanel {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths: paths.into_iter().take(MAX_APPS).collect(),
            selection: 0,
            busy: false,
            error: None,
            task: None,
            scroll: ScrollHandle::new(),
        }
    }
}

impl Island {
    pub(crate) fn add_launcher_app(&mut self, cx: &mut Context<Self>) {
        if self.launcher.busy {
            return;
        }
        if self.launcher.paths.len() == MAX_APPS {
            self.launcher.error = Some("Можно закрепить до 12 приложений. Удалите лишнее.".into());
            cx.notify();
            return;
        }
        let dialog = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Выберите приложение .exe".into()),
        });
        self.launcher.busy = true;
        let executor = cx.background_executor().clone();
        self.launcher.task = Some(cx.spawn(async move |this, cx| {
            let result = match dialog.await {
                Ok(Ok(Some(paths))) => {
                    executor
                        .spawn(async move {
                            paths
                                .first()
                                .map(|path| islandvolt_platform_windows::launcher::validate(path))
                                .transpose()
                        })
                        .await
                }
                Ok(Ok(None)) => Ok(None),
                _ => Err("Не удалось открыть выбор приложения".into()),
            };
            let _ = this.update(cx, |island, cx| {
                island.launcher.busy = false;
                match result {
                    Ok(Some(path)) => {
                        if !island.launcher.paths.contains(&path) {
                            island.launcher.paths.push(path);
                            island.settings.save_launcher(island.launcher.paths.clone());
                        }
                        island.launcher.error = None;
                    }
                    Ok(None) => {}
                    Err(error) => island.launcher.error = Some(error),
                }
                if !island.hover.inside {
                    island.hover_changed(false, cx);
                }
                cx.notify();
            });
        }));
    }

    pub(crate) fn launch_app(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.launcher.busy {
            return;
        }
        let Some(path) = self.launcher.paths.get(index).cloned() else {
            return;
        };
        self.launcher.selection = index;
        self.launcher.busy = true;
        let job = cx
            .background_executor()
            .spawn(async move { islandvolt_platform_windows::launcher::launch(&path) });
        self.launcher.task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            let _ = this.update(cx, |island, cx| {
                island.launcher.busy = false;
                island.launcher.error = result.err();
                if !island.hover.inside {
                    island.hover_changed(false, cx);
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(crate) fn unpin_app(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.launcher.busy || index >= self.launcher.paths.len() {
            return;
        }
        self.launcher.paths.remove(index);
        self.launcher.selection = self
            .launcher
            .selection
            .min(self.launcher.paths.len().saturating_sub(1));
        self.settings.save_launcher(self.launcher.paths.clone());
        self.launcher.error = None;
        cx.notify();
    }

    pub(crate) fn launcher_key(&mut self, key: &str, cx: &mut Context<Self>) -> bool {
        if !self.show_launcher {
            return false;
        }
        match key {
            "a" => self.add_launcher_app(cx),
            "escape" => {
                self.show_launcher = false;
                self.sync_pet(cx);
            }
            "up" => self.launcher.selection = self.launcher.selection.saturating_sub(1),
            "down" => {
                self.launcher.selection =
                    (self.launcher.selection + 1).min(self.launcher.paths.len().saturating_sub(1))
            }
            "enter" => self.launch_app(self.launcher.selection, cx),
            "delete" => self.unpin_app(self.launcher.selection, cx),
            _ => return false,
        }
        self.launcher.scroll.scroll_to_item(self.launcher.selection);
        cx.notify();
        true
    }
}
