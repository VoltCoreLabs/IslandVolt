//! GPU-tessellated notch, open against the screen's top edge.
use gpui::{Bounds, IntoElement, PathBuilder, Pixels, canvas, point, prelude::*, px, rgb};

pub fn background() -> impl IntoElement {
    canvas(
        |_, _, _| {},
        |bounds, _, window, _| {
            let mut fill = PathBuilder::fill();
            contour(&mut fill, bounds);
            fill.close();
            match fill.build() {
                Ok(path) => window.paint_path(path, rgb(0x030305)),
                Err(error) => eprintln!("Не удалось построить контур острова: {error}"),
            }
            let mut edge = PathBuilder::stroke(px(1.0));
            contour(&mut edge, bounds);
            if let Ok(path) = edge.build() {
                window.paint_path(path, rgb(0x39323e));
            }
        },
    )
    .absolute()
    .size_full()
}

fn contour(builder: &mut PathBuilder, bounds: Bounds<Pixels>) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let r = 25.0_f32.min(h / 2.0);
    let p = |x, y| bounds.origin + point(px(x), px(y));
    builder.move_to(p(0.0, 0.0));
    // Concave shoulder, then a generous convex bottom corner.
    builder.cubic_bezier_to(p(r, r), p(r, 0.0), p(r, r * 0.45));
    builder.cubic_bezier_to(p(r * 2.0, h - 5.0), p(r, h - 9.0), p(r * 1.15, h - 5.0));
    // A restrained liquid edge, not a fabricated audio waveform.
    builder.cubic_bezier_to(
        p(w * 0.5, h - 4.0),
        p(w * 0.30, h - 10.0),
        p(w * 0.33, h - 1.0),
    );
    builder.cubic_bezier_to(
        p(w - r * 2.0, h - 5.0),
        p(w * 0.67, h - 7.0),
        p(w * 0.70, h - 1.0),
    );
    builder.cubic_bezier_to(p(w - r, r), p(w - r * 1.15, h - 5.0), p(w - r, h - 9.0));
    builder.cubic_bezier_to(p(w, 0.0), p(w - r, r * 0.45), p(w - r, 0.0));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_and_expanded_paths_can_be_tessellated() {
        for wide in [false, true] {
            for progress in [0.0, 0.25, 0.5, 1.0] {
                let (w, h) = crate::layout::dimensions(wide, progress);
                let mut builder = PathBuilder::fill();
                contour(
                    &mut builder,
                    Bounds {
                        origin: point(px(0.0), px(0.0)),
                        size: gpui::size(px(w), px(h)),
                    },
                );
                builder.close();
                assert!(builder.build().is_ok());
            }
        }
    }
}
