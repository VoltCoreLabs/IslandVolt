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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub playing: bool,
}

pub struct MediaMonitor {
    signals: async_channel::Sender<()>,
}

impl Drop for MediaMonitor {
    fn drop(&mut self) {
        self.signals.close();
    }
}

pub fn watch() -> (
    MediaMonitor,
    async_channel::Receiver<Result<Option<Track>, String>>,
) {
    let (signals, events) = async_channel::bounded(1);
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
    events: &async_channel::Receiver<()>,
    notify: &async_channel::Sender<()>,
    updates: &async_channel::Sender<Result<Option<Track>, String>>,
) -> windows::core::Result<()> {
    let manager = Manager::RequestAsync()?.get()?;
    let manager_signal = notify.clone();
    let manager_token = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
        let _ = manager_signal.try_send(());
        Ok(())
    }))?;
    let mut subscribed: Option<(Session, i64, i64)> = None;
    let _ = notify.try_send(());
    while events.recv_blocking().is_ok() {
        let current = manager.GetCurrentSession().ok();
        if subscribed.as_ref().map(|entry| &entry.0) != current.as_ref() {
            unsubscribe(subscribed.take());
            if let Some(session) = current.as_ref() {
                let properties_signal = notify.clone();
                let properties =
                    session.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
                        let _ = properties_signal.try_send(());
                        Ok(())
                    }));
                let playback_signal = notify.clone();
                let playback = session.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
                    let _ = playback_signal.try_send(());
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
        let track = current.and_then(|session| read_track(&session).ok());
        if updates.try_send(Ok(track)).is_err() && updates.is_closed() {
            break;
        }
    }
    unsubscribe(subscribed);
    let _ = manager.RemoveCurrentSessionChanged(manager_token);
    Ok(())
}

fn unsubscribe(subscription: Option<(Session, i64, i64)>) {
    if let Some((session, properties, playback)) = subscription {
        let _ = session.RemoveMediaPropertiesChanged(properties);
        let _ = session.RemovePlaybackInfoChanged(playback);
    }
}

fn read_track(session: &Session) -> windows::core::Result<Track> {
    let properties = session.TryGetMediaPropertiesAsync()?.get()?;
    Ok(Track {
        title: properties.Title()?.to_string_lossy(),
        artist: properties.Artist()?.to_string_lossy(),
        playing: session.GetPlaybackInfo()?.PlaybackStatus()? == PlaybackStatus::Playing,
    })
}
