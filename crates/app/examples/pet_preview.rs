//! Render built-in artwork at UI size and at 3x for asset review, without a GPU/window.
fn main() -> anyhow::Result<()> {
    let output = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Укажите путь PNG для предпросмотра питомца"))?;
    let pets: [[&[u8]; 5]; 3] = [
        [
            include_bytes!("../../../assets/pets/volt-cat/idle.svg"),
            include_bytes!("../../../assets/pets/volt-cat/blink.svg"),
            include_bytes!("../../../assets/pets/volt-cat/happy.svg"),
            include_bytes!("../../../assets/pets/volt-cat/happy-up.svg"),
            include_bytes!("../../../assets/pets/volt-cat/sleep.svg"),
        ],
        [
            include_bytes!("../../../assets/pets/ember-fox/idle.svg"),
            include_bytes!("../../../assets/pets/ember-fox/blink.svg"),
            include_bytes!("../../../assets/pets/ember-fox/happy.svg"),
            include_bytes!("../../../assets/pets/ember-fox/happy-up.svg"),
            include_bytes!("../../../assets/pets/ember-fox/sleep.svg"),
        ],
        [
            include_bytes!("../../../assets/pets/mint-rabbit/idle.svg"),
            include_bytes!("../../../assets/pets/mint-rabbit/blink.svg"),
            include_bytes!("../../../assets/pets/mint-rabbit/happy.svg"),
            include_bytes!("../../../assets/pets/mint-rabbit/happy-up.svg"),
            include_bytes!("../../../assets/pets/mint-rabbit/sleep.svg"),
        ],
    ];
    let mut preview = resvg::tiny_skia::Pixmap::new(480, 300)
        .ok_or_else(|| anyhow::anyhow!("Не удалось создать предпросмотр"))?;
    preview.fill(resvg::tiny_skia::Color::from_rgba8(3, 3, 5, 255));
    for (row, frames) in pets.iter().enumerate() {
        for (column, frame) in frames.iter().enumerate() {
            let tree = resvg::usvg::Tree::from_data(frame, &resvg::usvg::Options::default())?;
            let scale = 72.0 / tree.size().width();
            let transform = resvg::tiny_skia::Transform::from_scale(scale, scale)
                .post_translate(column as f32 * 96.0 + 12.0, row as f32 * 100.0 + 14.0);
            resvg::render(&tree, transform, &mut preview.as_mut());
        }
    }
    preview.save_png(output)?;
    Ok(())
}
