//! GPU-tessellated notch, open against the screen's top edge.
use gpui::{Bounds, IntoElement, PathBuilder, Pixels, canvas, point, prelude::*, px, rgb};

pub fn background(width: f32, height: f32) -> impl IntoElement {
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
        },
    )
    .absolute()
    .w(px(width))
    .h(px(height))
}

fn contour(builder: &mut PathBuilder, bounds: Bounds<Pixels>) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let r = 25.0_f32.min(h / 2.0);
    let p = |x, y| bounds.origin + point(px(x), px(y));
    builder.move_to(p(0.0, 0.0));
    let bottom = 18.0_f32.min((h - r) / 2.0);
    let tangent = bottom * (1.0 - 0.552_284_8);
    // Concave top shoulder, vertical side, and a quarter-circle bottom corner.
    builder.cubic_bezier_to(p(r, r), p(r, 0.0), p(r, r * 0.45));
    builder.line_to(p(r, h - bottom));
    builder.cubic_bezier_to(p(r + bottom, h), p(r, h - tangent), p(r + tangent, h));
    builder.line_to(p(w - r - bottom, h));
    builder.cubic_bezier_to(
        p(w - r, h - bottom),
        p(w - r - tangent, h),
        p(w - r, h - tangent),
    );
    builder.line_to(p(w - r, r));
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
