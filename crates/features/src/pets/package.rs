use serde::Deserialize;
use std::collections::BTreeMap;

/// Visual states supported by the first built-in package format.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Idle,
    Blink,
    Happy,
    Sleep,
}

/// One embedded SVG frame and its duration in milliseconds.
#[derive(Debug, Deserialize)]
pub struct Frame {
    pub asset: String,
    pub duration_ms: u64,
}

/// A bounded, non-executable description of a built-in pet.
#[derive(Debug)]
pub struct Package(PackageData);

#[derive(Debug, Deserialize)]
struct PackageData {
    schema_version: u32,
    id: String,
    name: String,
    author: String,
    license: String,
    size: u32,
    sleep_after_ms: u64,
    animations: BTreeMap<State, Vec<Frame>>,
}

impl Package {
    /// Parse bounded metadata; paths are relative asset IDs, never filesystem paths.
    pub fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > 32_768 {
            return Err("Описание питомца превышает 32 КиБ".into());
        }
        let data: PackageData = serde_json::from_str(json)
            .map_err(|error| format!("Не удалось прочитать описание питомца: {error}"))?;
        let package = Self(data);
        package.validate()?;
        Ok(package)
    }

    fn validate(&self) -> Result<(), String> {
        let data = &self.0;
        if data.schema_version != 1 {
            return Err("Версия пакета питомца не поддерживается".into());
        }
        if data.id.is_empty()
            || data.id.len() > 64
            || !data
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || [&data.name, &data.author, &data.license]
                .iter()
                .any(|value| value.trim().is_empty() || value.len() > 128)
        {
            return Err("Проверьте ID, имя, автора и лицензию питомца".into());
        }
        if !(16..=128).contains(&data.size)
            || !(5_000..=3_600_000).contains(&data.sleep_after_ms)
            || !data.animations.contains_key(&State::Idle)
        {
            return Err("Пакету нужны idle, размер 16–128 и корректный таймер сна".into());
        }
        for frames in data.animations.values() {
            if frames.is_empty() || frames.len() > 32 {
                return Err("Анимация должна содержать от 1 до 32 кадров".into());
            }
            for frame in frames {
                if !(50..=60_000).contains(&frame.duration_ms) || !safe_asset(&frame.asset) {
                    return Err("Проверьте длительность кадра и относительный путь SVG".into());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn name(&self) -> &str {
        &self.0.name
    }

    pub(crate) fn sleep_after_ms(&self) -> u64 {
        self.0.sleep_after_ms
    }

    pub(crate) fn frames(&self, state: State) -> &[Frame] {
        self.0
            .animations
            .get(&state)
            .or_else(|| self.0.animations.get(&State::Idle))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

fn safe_asset(asset: &str) -> bool {
    asset.len() <= 128
        && asset.starts_with("pets/")
        && asset.ends_with(".svg")
        && asset
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && asset
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/-_.".contains(&c))
}
