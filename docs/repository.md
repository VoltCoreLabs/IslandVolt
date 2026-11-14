# Настройка репозитория IslandVolt

## Идентичность

- Продукт: **IslandVolt**.
- Репозиторий: [VoltCoreLabs/IslandVolt](https://github.com/VoltCoreLabs/IslandVolt).
- Префикс будущих Rust-пакетов: `islandvolt-`.
- Основная ветка: `main`.
- Лицензия: [MIT](../LICENSE), выбрана владельцем.
- Текущий этап: технический прототип GPUI; полноценное приложение ещё не реализовано.

Локальный путь рабочей папки может отличаться от имени продукта; Git remote использует
канонический URL. Перемещение открытой рабочей папки не требуется для переименования проекта.

## About

Описание GitHub:

> A customizable Dynamic Island for Windows 10/11, built with Rust + GPUI.
> Planned: media and audio controls, screen tools, notes, AI assistance, and animated companions.

Темы: `rust`, `gpui`, `windows`, `windows-10`, `windows-11`, `desktop-app`, `dynamic-island`,
`productivity`, `audio-control`, `screen-capture`, `ai-assistant`, `animated-pets`.
Темы обозначают направление проекта, а не готовность перечисленных функций.
Отдельного сайта пока нет, поэтому поле homepage оставлено пустым.

## Работа с изменениями

- Issues и Projects включены; документация находится в репозитории, Wiki выключена.
- Discussions пока не используются: обсуждения ведутся в Issues и PR.
- Единственный метод merge — squash, заголовок и описание берутся из PR.
- После merge GitHub автоматически удаляет рабочую ветку.
- Создание PR и merge выполняются самостоятельно в рамках разрешения владельца.
- Шаблоны: ошибка, возможность, техническая задача; PR содержит результат и проверки.

Метки по типу: `bug`, `enhancement`, `documentation`, `task`, `question`.
Метки областей: `area:core`, `area:ui`, `area:windows`, `area:audio`, `area:pets`,
`area:ai`, `area:storage`, `area:tooling`.
Приоритеты: `priority:high`, `priority:normal`, `priority:low`.
Стандартные метки GitHub сохраняются, чтобы не нарушать существующие ссылки и привычки.

## Проверки

Workflow `Repository checks` запускается для PR, push в `main` и вручную.
Он проверяет лимит 300 строк, UTF-8, локальные inline-ссылки Markdown и старые имена проекта.
Удалённые ссылки, якоря и поведение приложения эта проверка не подтверждает.
Запуск локально: `python tools/check_repository.py`.
Workflow имеет только чтение содержимого; checkout закреплён на commit, без сохранения credentials.
Cargo-проверки добавляются после начала реализации Rust-приложения.

## Ограничение защиты ветки

При настройке 2026-11-14 GitHub API вернул HTTP 403 для branch protection и rulesets:
для приватного репозитория требуется подходящий тариф либо публичная видимость.
Видимость не менялась. Защита `main` сервером сейчас не обеспечивается.
Правило команды: только PR, успешный CI и squash merge; не применять force-push.
Когда тариф позволит, включить PR + обязательный `Repository checks`, запрет удаления
и force-push; обязательное одобрение второго участника согласовать с составом команды.

## Будущие настройки

Релизы и Git-теги создаются для реальных версий, а не для пустого каркаса.
Установщик, подпись, release pipeline и проверки Rust добавляются по roadmap.
Социальное превью и иконка появятся после утверждения визуального стиля.
