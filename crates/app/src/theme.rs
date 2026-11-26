//! Island color roles shared by the shell and all its panels.
use gpui::{Rgba, WindowAppearance, rgb};
use serde::{Deserialize, Serialize};

/// Persisted appearance preference. Older settings retain the dark island.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    #[default]
    #[serde(other)]
    Dark,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::System,
            Self::System => Self::Dark,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Тёмная",
            Self::Light => "Светлая",
            Self::System => "Как в Windows",
        }
    }

    pub fn palette(self, system: WindowAppearance) -> Palette {
        let light = match self {
            Self::Light => true,
            Self::Dark => false,
            Self::System => matches!(
                system,
                WindowAppearance::Light | WindowAppearance::VibrantLight
            ),
        };
        if light {
            Palette::light()
        } else {
            Palette::dark()
        }
    }
}

impl crate::island::Island {
    pub(crate) fn cycle_theme(&mut self, cx: &mut gpui::Context<Self>) {
        self.theme = self.theme.next();
        self.settings.save_theme(self.theme);
        cx.notify();
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub shell: Rgba,
    pub surface: Rgba,
    pub selected: Rgba,
    pub text: Rgba,
    pub secondary: Rgba,
    pub disabled: Rgba,
    pub accent: Rgba,
    pub slider_track: Rgba,
    pub slider_thumb: Rgba,
    pub warning: Rgba,
}

impl Palette {
    fn dark() -> Self {
        Self {
            shell: rgb(0x030305),
            surface: rgb(0x24202a),
            selected: rgb(0x30283b),
            text: rgb(0xf5f6fa),
            secondary: rgb(0xb4b6c2),
            disabled: rgb(0x858793),
            accent: rgb(0xcabaff),
            slider_track: rgb(0x373943),
            slider_thumb: rgb(0xededf2),
            warning: rgb(0xf5b9a1),
        }
    }

    fn light() -> Self {
        Self {
            shell: rgb(0xf7f3fa),
            surface: rgb(0xede6f1),
            selected: rgb(0xeae2fa),
            text: rgb(0x221b2c),
            secondary: rgb(0x575064),
            disabled: rgb(0x686176),
            accent: rgb(0x5838a4),
            slider_track: rgb(0xc8bfd3),
            slider_thumb: rgb(0x302047),
            warning: rgb(0x912e24),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_system_mode_uses_window_appearance() {
        use WindowAppearance::*;
        assert_eq!(
            ThemeMode::System.palette(VibrantLight).shell,
            Palette::light().shell
        );
        assert_eq!(
            ThemeMode::System.palette(VibrantDark).shell,
            Palette::dark().shell
        );
        assert_eq!(ThemeMode::Dark.palette(Light).shell, Palette::dark().shell);
        assert_eq!(ThemeMode::Light.palette(Dark).shell, Palette::light().shell);
    }

    #[test]
    fn small_text_has_contrast_on_both_themes() {
        for palette in [Palette::dark(), Palette::light()] {
            for text in [
                palette.text,
                palette.secondary,
                palette.accent,
                palette.warning,
            ] {
                for background in [palette.shell, palette.surface, palette.selected] {
                    let (text, bg) = (luminance(text), luminance(background));
                    let contrast = (text.max(bg) + 0.05) / (text.min(bg) + 0.05);
                    assert!(contrast >= 4.5, "contrast {contrast}");
                }
            }
        }
    }

    fn luminance(color: Rgba) -> f32 {
        let linear = |c: f32| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
}
