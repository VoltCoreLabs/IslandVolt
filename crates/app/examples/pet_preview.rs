//! Render built-in artwork at UI size and at 3x for asset review, without a GPU/window.
fn main() -> anyhow::Result<()> {
    let output = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Укажите путь PNG для предпросмотра питомца"))?;
    let frames: [&[u8]; 5] = [
        include_bytes!("../../../assets/pets/volt-cat/idle.svg"),
        include_bytes!("../../../assets/pets/volt-cat/blink.svg"),
        include_bytes!("../../../assets/pets/volt-cat/happy.svg"),
        include_bytes!("../../../assets/pets/volt-cat/happy-up.svg"),
        include_bytes!("../../../assets/pets/volt-cat/sleep.svg"),
    ];
    let mut preview = resvg::tiny_skia::Pixmap::new(600, 200)
        .ok_or_else(|| anyhow::anyhow!("Не удалось создать предпросмотр"))?;
    preview.fill(resvg::tiny_skia::Color::from_rgba8(3, 3, 5, 255));
    for (index, frame) in frames.iter().enumerate() {
        let tree = resvg::usvg::Tree::from_data(frame, &resvg::usvg::Options::default())?;
        let x = index as f32 * 120.0;
        for (size, y) in [(32.0, 15.0), (96.0, 75.0)] {
            let transform = resvg::tiny_skia::Transform::from_scale(size / 48.0, size / 48.0)
                .post_translate(x + (120.0 - size) / 2.0, y);
            resvg::render(&tree, transform, &mut preview.as_mut());
        }
    }
    preview.save_png(output)?;
    Ok(())
}
