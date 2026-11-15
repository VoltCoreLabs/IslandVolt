/// Hover changes height only; horizontal position stays anchored to monitor center.
pub fn dimensions(wide: bool, progress: f32) -> (f32, f32) {
    (
        if wide { 540.0 } else { 460.0 },
        64.0 + 196.0 * progress.clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn hover_expansion_keeps_width_constant() {
        for wide in [false, true] {
            let compact = super::dimensions(wide, 0.0);
            for progress in [0.25, 0.5, 0.75, 1.0] {
                let expanded = super::dimensions(wide, progress);
                assert_eq!(compact.0, expanded.0);
                assert!(expanded.1 > compact.1);
            }
        }
    }
}
