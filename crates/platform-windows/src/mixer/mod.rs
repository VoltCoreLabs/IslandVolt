//! On-demand mixer for shared audio sessions on the default output device.
mod callbacks;
mod names;
mod sessions;
mod worker;

use crate::volume::VolumeState;
use async_channel::{Receiver, Sender};

/// An audio session. An app can own more than one session.
#[derive(Clone, Debug)]
pub struct Session {
    pub id: u64,
    pub name: String,
    pub volume: VolumeState,
}

/// The latest view of the default output. Warnings can accompany valid rows.
#[derive(Default)]
pub struct Snapshot {
    pub output: Option<String>,
    pub sessions: Vec<Session>,
    pub warning: Option<String>,
}

#[derive(Clone, Copy)]
pub enum Action {
    Set(u8),
    Step(i8),
    ToggleMute,
}

pub(super) enum Message {
    Refresh,
    Adjust(u64, Action),
}

/// The worker stops and releases its native subscriptions when this handle is dropped.
pub struct Mixer {
    commands: Sender<Message>,
}

impl Mixer {
    pub fn adjust(&self, id: u64, action: Action) -> Result<(), &'static str> {
        self.send(Message::Adjust(id, action))
    }

    pub fn refresh(&self) -> Result<(), &'static str> {
        self.send(Message::Refresh)
    }

    fn send(&self, message: Message) -> Result<(), &'static str> {
        self.commands
            .try_send(message)
            .map_err(|_| "Микшер занят. Попробуйте ещё раз.")
    }
}

impl Drop for Mixer {
    fn drop(&mut self) {
        self.commands.close();
    }
}

/// Start the mixer on a dedicated COM thread and return a bounded snapshot stream.
pub fn open() -> (Mixer, Receiver<Snapshot>) {
    let (commands, events) = async_channel::bounded(16);
    let (updates, receiver) = async_channel::bounded(1);
    worker::start(events, commands.clone(), updates);
    (Mixer { commands }, receiver)
}
