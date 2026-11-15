use super::{Manager, Session, Source};

pub(super) struct Sessions {
    entries: Vec<(u64, Session)>,
    next_id: u64,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    pub fn refresh(&mut self, manager: &Manager) -> windows::core::Result<()> {
        let sessions = manager.GetSessions()?;
        let mut entries = Vec::new();
        for index in 0..sessions.Size()? {
            let session = sessions.GetAt(index)?;
            let id = if let Some((id, _)) = self.entries.iter().find(|(_, old)| old == &session) {
                *id
            } else {
                let id = self.next_id;
                self.next_id += 1;
                id
            };
            entries.push((id, session));
        }
        self.entries = entries;
        Ok(())
    }

    pub fn contains(&self, id: u64) -> bool {
        self.entries.iter().any(|(key, _)| *key == id)
    }

    pub fn choose(&self, selected: Option<u64>, manager: &Manager) -> Option<(u64, Session)> {
        let automatic = manager.GetCurrentSession().ok();
        self.entries
            .iter()
            .find(|(id, session)| {
                selected.map_or_else(
                    || automatic.as_ref() == Some(session),
                    |selected| selected == *id,
                )
            })
            .or_else(|| {
                if selected.is_none() {
                    self.entries.first()
                } else {
                    None
                }
            })
            .cloned()
    }

    pub fn sources(&self) -> Vec<Source> {
        self.entries
            .iter()
            .map(|(id, session)| Source {
                id: *id,
                name: session
                    .SourceAppUserModelId()
                    .map(|id| id.to_string_lossy())
                    .unwrap_or_else(|_| "Плеер".into()),
            })
            .collect()
    }
}
