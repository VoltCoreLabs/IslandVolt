use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};

/// Playback command sent to the selected system media session.
#[derive(Clone, Copy, Debug)]
pub enum MediaAction {
    Toggle,
    Previous,
    Next,
}

pub(super) fn execute(session: &Session, action: MediaAction) -> Result<(), String> {
    let info = session
        .GetPlaybackInfo()
        .map_err(|error| error.to_string())?;
    let controls = info.Controls().map_err(|error| error.to_string())?;
    let playing =
        info.PlaybackStatus().map_err(|error| error.to_string())? == PlaybackStatus::Playing;
    let accepted = match action {
        MediaAction::Toggle if playing && controls.IsPauseEnabled().unwrap_or(false) => session
            .TryPauseAsync()
            .and_then(|operation| operation.get()),
        MediaAction::Toggle if !playing && controls.IsPlayEnabled().unwrap_or(false) => {
            session.TryPlayAsync().and_then(|operation| operation.get())
        }
        MediaAction::Previous if controls.IsPreviousEnabled().unwrap_or(false) => session
            .TrySkipPreviousAsync()
            .and_then(|operation| operation.get()),
        MediaAction::Next if controls.IsNextEnabled().unwrap_or(false) => session
            .TrySkipNextAsync()
            .and_then(|operation| operation.get()),
        _ => return Err("Плеер не поддерживает эту команду".into()),
    }
    .map_err(|error| error.to_string())?;
    if accepted {
        Ok(())
    } else {
        Err("Плеер отклонил команду".into())
    }
}
