# Структура каталогов

Каталоги ниже создаются сейчас как каркас. `src`, Rust-файлы и `Cargo.toml` появятся
после перехода к реализации. Пустые каталоги сохраняются в репозитории через `.gitkeep`,
чтобы каркас был доступен после clone. При появлении содержимого placeholder удаляется.

```text
inslandio/
├─ AGENTS.md
├─ README.md
├─ docs/
│  ├─ product.md
│  ├─ architecture.md
│  ├─ structure.md
│  ├─ island-ux.md
│  ├─ platform-windows.md
│  ├─ data-and-permissions.md
│  ├─ pets.md
│  ├─ roadmap.md
│  └─ decisions.md
├─ crates/
│  ├─ app/
│  ├─ core/
│  ├─ features/
│  │  ├─ island/
│  │  ├─ media/
│  │  ├─ audio/
│  │  ├─ clock/
│  │  ├─ pets/
│  │  ├─ settings/
│  │  ├─ launcher/
│  │  ├─ input_language/
│  │  ├─ clipboard/
│  │  ├─ notes/
│  │  ├─ reminders/
│  │  ├─ capture/
│  │  ├─ ocr/
│  │  ├─ qr/
│  │  ├─ color_picker/
│  │  ├─ recording/
│  │  ├─ recognition/
│  │  ├─ speech/
│  │  ├─ lyrics/
│  │  ├─ weather/
│  │  ├─ system_controls/
│  │  └─ assistant/
│  ├─ ui/
│  │  ├─ shell/
│  │  ├─ cards/
│  │  ├─ components/
│  │  ├─ theme/
│  │  └─ animation/
│  ├─ platform-windows/
│  │  ├─ window/
│  │  ├─ audio/
│  │  ├─ media/
│  │  ├─ capture/
│  │  ├─ clipboard/
│  │  ├─ input/
│  │  └─ system/
│  ├─ storage/
│  └─ providers/
│     ├─ ai/
│     ├─ speech/
│     ├─ recognition/
│     ├─ lyrics/
│     └─ weather/
├─ assets/
│  ├─ pets/
│  ├─ icons/
│  ├─ fonts/
│  └─ sounds/
├─ tests/
│  ├─ integration/
│  └─ fixtures/
└─ tools/
```

## Преобразование в Cargo workspace

При реализации каталоги с модулями переносятся под `src` соответствующего пакета.
Например, `crates/features/media/` станет `crates/features/src/media/`.
Это не отдельный crate; Cargo-манифесты нужны только семи пакетам из архитектуры.
Пустые будущие модули не подключаются и не получают фиктивных заглушек.
Корневые manifest, lockfile и toolchain создаются после проверенной сборки GPUI.

## Типовая декомпозиция фичи при реализации

```text
media/
├─ mod.rs          # публичная поверхность
├─ model.rs        # предметные типы
├─ state.rs        # состояние и переходы
├─ commands.rs     # команды пользователя
├─ service.rs      # сценарии и управление задачами
└─ tests.rs        # проверка значимого поведения
```

Создавать только нужные файлы. Контракты внешних интеграций — в тематических портах core;
реализация Windows — в `platform-windows`, карточка GPUI — в `ui/cards/media`.
Не добавлять одинаковые пустые файлы по шаблону во все фичи.

## Где размещать новые части

| Изменение | Место |
| --- | --- |
| Новая предметная возможность | `features/<name>` |
| Карточка и её presentation-модель | `ui/cards/<name>` |
| Общая кнопка, slider, меню | `ui/components` |
| Цвета, размеры, текстовые стили | `ui/theme` |
| Общая анимационная механика | `ui/animation` |
| Новый Windows API | тематический модуль `platform-windows` |
| Новый сетевой/локальный провайдер | тематический модуль `providers` |
| Миграция или репозиторий | `storage` |
| Набор кадров и метаданные питомца | `assets/pets/<pet-id>` |
| Обоснование решения | `docs/decisions.md`, далее отдельный ADR при росте |
| Автоматизация разработчика | `tools` |

## Лимит 300 строк

Если `service.rs` растёт, выделить конкретные сценарии: `playback.rs`, `selection.rs`.
Если `model.rs` растёт, выделить самостоятельные типы по предметному смыслу.
Файл `mod.rs` должен оставаться навигацией и публичным API.
Большие JSON-пакеты животных делятся на манифест и описания анимаций.
При росте документа разделить его по теме и добавить ссылки в оглавление.
