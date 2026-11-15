use super::{Frame, Package, State};
use std::time::Duration;

/// Deterministic playback: time only advances when the owner delivers a frame deadline.
pub struct Animator {
    package: Package,
    state: State,
    frame: usize,
    quiet_ms: u64,
    reduced_motion: bool,
    visible: bool,
}

impl Animator {
    /// Start a validated package, using a static sleep frame for reduced motion.
    pub fn new(package: Package, reduced_motion: bool) -> Self {
        Self {
            package,
            state: if reduced_motion {
                State::Sleep
            } else {
                State::Idle
            },
            frame: 0,
            quiet_ms: 0,
            reduced_motion,
            visible: true,
        }
    }

    /// Current visual state; missing optional artwork falls back to idle.
    pub fn state(&self) -> State {
        self.state
    }

    /// Display name supplied by the validated package.
    pub fn name(&self) -> &str {
        self.package.name()
    }

    /// Current frame, including fallback artwork for missing optional states.
    pub fn frame(&self) -> &Frame {
        &self.package.frames(self.state)[self.frame]
    }

    /// Pause deadlines and state advancement while hidden; returns whether visibility changed.
    pub fn set_visible(&mut self, visible: bool) -> bool {
        let changed = self.visible != visible;
        self.visible = visible;
        changed
    }

    /// Sleeping/reduced-motion pets do not request timer wakeups.
    pub fn next_delay(&self) -> Option<Duration> {
        if !self.visible || self.reduced_motion || self.state == State::Sleep {
            None
        } else {
            Some(Duration::from_millis(self.frame().duration_ms))
        }
    }

    /// Advance exactly one frame. The UI cancels deadlines while the pet is hidden.
    pub fn advance(&mut self) {
        if self.next_delay().is_none() {
            return;
        }
        if self.state != State::Happy {
            self.quiet_ms = self.quiet_ms.saturating_add(self.frame().duration_ms);
        }
        self.frame += 1;
        if self.quiet_ms >= self.package.sleep_after_ms() {
            self.state = State::Sleep;
            self.frame = 0;
        } else if self.frame >= self.package.frames(self.state).len() {
            self.state = match self.state {
                State::Idle => State::Blink,
                _ => State::Idle,
            };
            self.frame = 0;
        }
    }

    /// A user click wakes the pet and restarts its short happy reaction.
    pub fn react(&mut self) {
        if self.visible && !self.reduced_motion {
            self.state = State::Happy;
            self.frame = 0;
            self.quiet_ms = 0;
        }
    }
}
