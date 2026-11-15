use windows::Win32::Foundation::POINT;

/// Top-edge notch: flared shoulders, straight sides, rounded bottom corners.
pub(crate) fn points(width: i32, height: i32, radius: i32) -> Vec<POINT> {
    let w = width as f32;
    let h = height as f32;
    let r = radius.min(height / 2).min(width / 4).max(1) as f32;
    let mut result = vec![POINT { x: 0, y: 0 }];
    let mut curve = |a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32)| {
        for step in 1..=16 {
            let t = step as f32 / 16.0;
            let u = 1.0 - t;
            result.push(POINT {
                x: (u * u * u * a.0
                    + 3.0 * u * u * t * b.0
                    + 3.0 * u * t * t * c.0
                    + t * t * t * d.0)
                    .round() as i32,
                y: (u * u * u * a.1
                    + 3.0 * u * u * t * b.1
                    + 3.0 * u * t * t * c.1
                    + t * t * t * d.1)
                    .round() as i32,
            });
        }
    };
    curve((0.0, 0.0), (r, 0.0), (r, r * 0.5), (r, r));
    curve((r, r), (r, h - r), (r, h), (2.0 * r, h));
    curve((2.0 * r, h), (w / 2.0, h), (w / 2.0, h), (w - 2.0 * r, h));
    curve((w - 2.0 * r, h), (w - r, h), (w - r, h - r), (w - r, r));
    curve((w - r, r), (w - r, r * 0.5), (w - r, 0.0), (w, 0.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outline_touches_top_and_stays_in_bounds() {
        let shape = points(400, 76, 24);
        assert_eq!((shape[0].x, shape[0].y), (0, 0));
        let last = shape.last().expect("outline");
        assert_eq!((last.x, last.y), (400, 0));
        assert!(
            shape
                .iter()
                .all(|p| (0..=400).contains(&p.x) && (0..=76).contains(&p.y))
        );
        assert!(shape.iter().any(|p| p.y == 76 && p.x > 0 && p.x < 400));
    }
}
