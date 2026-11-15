//! Event-driven Windows media session monitor.

use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
    },
    Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
};

mod controls;
mod sessions;
pub use controls::MediaAction;

#[derive(Clone, Debug)]
pub struct Source {
    pub id: u64,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct MediaState {
    pub track: Option<Track>,
    pub sources: Vec<Source>,
    pub selected: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub can_toggle: bool,
    pub can_previous: bool,
    pub can_next: bool,
    pub session_id: u64,
}

#[derive(Clone, Copy)]
enum Message {
    Refresh,
    Control(u64, MediaAction),
    Select(Option<u64>),
}

pub struct MediaMonitor {
    signals: async_channel::Sender<Message>,
}

impl MediaMonitor {
    /// Select a stable session ID or return to Windows' automatic choice.
    pub fn select(&self, id: Option<u64>) -> Result<(), &'static str> {
        self.signals
            .try_send(Message::Select(id))
            .map_err(|_| "Не удалось выбрать плеер")
    }
    /// Queue an action for the session currently displayed by the UI.
    pub fn toggle(&self, session_id: u64) -> Result<(), &'static str> {
        self.control(session_id, MediaAction::Toggle)
    }

    /// Queue a media command for the session currently displayed by the UI.
    pub fn control(&self, session_id: u64, action: MediaAction) -> Result<(), &'static str> {
        self.signals
            .try_send(Message::Control(session_id, action))
            .map_err(|_| "Не удалось отправить команду плееру")
    }
}

impl Drop for MediaMonitor {
    fn drop(&mut self) {
        self.signals.close();
    }
}

pub fn watch() -> (
    MediaMonitor,
    async_channel::Receiver<Result<MediaState, String>>,
) {
    let (signals, events) = async_channel::bounded(8);
    let (updates, receiver) = async_channel::bounded(8);
    let notify = signals.clone();
    std::thread::spawn(move || {
        // SAFETY: COM is initialized and uninitialized on this dedicated thread.
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        if initialized {
            if let Err(error) = run(&events, &notify, &updates) {
                eprintln!("Медиасессия недоступна: {error}");
                let _ = updates.try_send(Err(error.to_string()));
            }
            // SAFETY: balances the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
        } else {
            let _ = updates.try_send(Err("Не удалось инициализировать COM".into()));
        }
    });
    (MediaMonitor { signals }, receiver)
}

fn run(
    events: &async_channel::Receiver<Message>,
    notify: &async_channel::Sender<Message>,
    updates: &async_channel::Sender<Result<MediaState, String>>,
) -> windows::core::Result<()> {
    let manager = Manager::RequestAsync()?.get()?;
    let manager_signal = notify.clone();
    let manager_token = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
        let _ = manager_signal.try_send(Message::Refresh);
        Ok(())
    }))?;
    let list_signal = notify.clone();
    let list_token = match manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
        let _ = list_signal.try_send(Message::Refresh);
        Ok(())
    })) {
        Ok(token) => token,
        Err(error) => {
            let _ = manager.RemoveCurrentSessionChanged(manager_token);
            return Err(error);
        }
    };
    let mut subscribed: Option<(Session, i64, i64)> = None;
    let mut sessions = sessions::Sessions::new();
    let mut selected = None;
    let _ = notify.try_send(Message::Refresh);
    while let Ok(message) = events.recv_blocking() {
        if let Err(error) = sessions.refresh(&manager) {
            let _ = updates.try_send(Err(error.to_string()));
            continue;
        }
        if let Message::Select(id) = message {
            if id.is_none_or(|id| sessions.contains(id)) {
                selected = id;
            } else {
                let _ = updates.try_send(Err("Плеер больше недоступен".into()));
            }
        }
        if selected.is_some_and(|id| !sessions.contains(id)) {
            selected = None;
        }
        let chosen = sessions.choose(selected, &manager);
        let session_id = chosen.as_ref().map_or(0, |(id, _)| *id);
        let current = chosen.map(|(_, session)| session);
        if subscribed.as_ref().map(|entry| &entry.0) != current.as_ref() {
            unsubscribe(subscribed.take());
            if let Some(session) = current.as_ref() {
                let properties_signal = notify.clone();
                let properties =
                    session.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
                        let _ = properties_signal.try_send(Message::Refresh);
                        Ok(())
                    }));
                let playback_signal = notify.clone();
                let playback = session.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
                    let _ = playback_signal.try_send(Message::Refresh);
                    Ok(())
                }));
                match (properties, playback) {
                    (Ok(properties), Ok(playback)) => {
                        subscribed = Some((session.clone(), properties, playback))
                    }
                    (Ok(properties), Err(_)) => {
                        let _ = session.RemoveMediaPropertiesChanged(properties);
                    }
                    (Err(_), Ok(playback)) => {
                        let _ = session.RemovePlaybackInfoChanged(playback);
                    }
                    (Err(_), Err(_)) => {}
                }
            }
        }
        if let Message::Control(expected, action) = message {
            let outcome = current
                .as_ref()
                .filter(|_| expected == session_id)
                .ok_or_else(|| "Медиасессия изменилась".to_string())
                .and_then(|session| controls::execute(session, action));
            if let Err(error) = outcome {
                let _ = updates.try_send(Err(error));
            }
        }
        let track = current.and_then(|session| read_track(&session, session_id).ok());
        let state = MediaState {
            track,
            sources: sessions.sources(),
            selected,
        };
        if updates.try_send(Ok(state)).is_err() && updates.is_closed() {
            break;
        }
    }
    unsubscribe(subscribed);
    let _ = manager.RemoveCurrentSessionChanged(manager_token);
    let _ = manager.RemoveSessionsChanged(list_token);
    Ok(())
}

fn unsubscribe(subscription: Option<(Session, i64, i64)>) {
    if let Some((session, properties, playback)) = subscription {
        let _ = session.RemoveMediaPropertiesChanged(properties);
        let _ = session.RemovePlaybackInfoChanged(playback);
    }
}

fn read_track(session: &Session, session_id: u64) -> windows::core::Result<Track> {
    let properties = session.TryGetMediaPropertiesAsync()?.get()?;
    let playback = session.GetPlaybackInfo()?;
    let playing = playback.PlaybackStatus()? == PlaybackStatus::Playing;
    let controls = playback.Controls()?;
    Ok(Track {
        title: properties.Title()?.to_string_lossy(),
        artist: properties.Artist()?.to_string_lossy(),
        playing,
        can_toggle: if playing {
            controls.IsPauseEnabled()?
        } else {
            controls.IsPlayEnabled()?
        },
        can_previous: controls.IsPreviousEnabled()?,
        can_next: controls.IsNextEnabled()?,
        session_id,
    })
}
