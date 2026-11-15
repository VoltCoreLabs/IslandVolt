use std::time::{Duration, Instant};

const DURATION: Duration = Duration::from_millis(180);

/// Interruptible interpolation between compact and expanded window sizes.
pub struct Transition {
    enabled: bool,
    from: f32,
    to: f32,
    started: Instant,
}

impl Transition {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            from: 0.0,
            to: 0.0,
            started: Instant::now(),
        }
    }

    pub fn set(&mut self, expanded: bool) {
        let target = if expanded { 1.0 } else { 0.0 };
        if !self.enabled {
            self.from = target;
            self.to = target;
            return;
        }
        if self.to != target {
            self.from = self.value();
            self.to = target;
            self.started = Instant::now();
        }
    }

    pub fn value(&self) -> f32 {
        let t = (self.started.elapsed().as_secs_f32() / DURATION.as_secs_f32()).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.from + (self.to - self.from) * eased
    }

    pub fn active(&self) -> bool {
        (self.value() - self.to).abs() > 0.001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversal_preserves_current_size() {
        let mut transition = Transition {
            enabled: true,
            from: 0.0,
            to: 1.0,
            started: Instant::now() - Duration::from_millis(90),
        };
        let before = transition.value();
        transition.set(false);
        assert!((transition.value() - before).abs() < 0.02);
        assert!(transition.active());
    }

    #[test]
    fn completed_transition_stops_requesting_frames() {
        let transition = Transition {
            enabled: true,
            from: 0.0,
            to: 1.0,
            started: Instant::now() - DURATION,
        };
        assert_eq!(transition.value(), 1.0);
        assert!(!transition.active());
    }
}
