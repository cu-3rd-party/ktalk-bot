# Contributing

Этот документ описывает, в какой фазе находится проект `ktalk-bot`, какие изменения сейчас наиболее полезны и какие границы важно соблюдать.

## Текущая фаза

Сейчас проект находится в ранней фазе развития headless-клиента:

- базовый cookie-based engine уже существует
- HTTP bootstrap работает
- system websocket и XMPP flow реализованы в рабочем приближении
- есть Python API через PyO3
- есть tooling для захвата сетевого трафика в браузере

Главная цель текущей фазы:
- добиться максимально точной и устойчивой протокольной совместимости с реальным браузерным клиентом

## Приоритеты текущей фазы

Сейчас особенно полезны вклады в следующие направления:

1. Протокольная точность

- уточнение HTTP / websocket / XMPP последовательностей по реальным capture-логам
- устранение guessed-поведения, если есть подтвержденный browser trace
- уточнение payload-полей, action names, keepalive, handshake

2. Надежность

- reconnect / retry logic
- корректный lifecycle завершения сессии
- диагностика ошибок
- идемпотентность engine-операций

3. Тестируемость

- fixture-based tests по реальным логам
- parser tests
- integration tests для ключевых сценариев

4. Инструменты reverse engineering

- улучшение Chrome extension
- нормализация и фильтрация trace-логов
- экспорт и анализ пакетов

## Что сейчас не стоит делать без отдельного обсуждения

Нежелательно вносить крупные изменения в эти области без явного архитектурного решения:

- полноценная WebRTC media-публикация
- сложный async Python API поверх текущего sync surface
- большой рефакторинг доменной модели без конкретной проблемы
- привязка логики к одному конкретному субдомену KTalk
- смешивание reverse-engineering tooling и production runtime в одном слое

## Архитектурные принципы

При внесении изменений придерживайтесь следующих правил:

- библиотека должна оставаться headless-first
- сетевой код должен быть максимально наблюдаемым и тестируемым
- source of truth для протокола — реальные packet captures, а не предположения
- доменная логика, orchestration и transport-level код должны оставаться разделенными
- Python API должен оставаться простым и понятным
- Node.js / TypeScript bindings через `src/interface/node.rs` должны оставаться тонкими и вызывать тот же Rust core, без дублирования protocol logic

## Как предлагать изменения

Желательный порядок работы:

1. Зафиксировать наблюдение

- приложить packet log, HAR, websocket trace или другой реальный артефакт

2. Сформулировать несоответствие

- что делает браузер
- что делает текущая библиотека
- где именно расходится поведение

3. Внести минимальное корректное изменение

- сначала делаем protocol parity
- потом рефакторим структуру, если это действительно нужно

4. Добавить тесты

- на parser-level
- на orchestration-level
- по возможности на основе реальных fixture-данных

## Требования к изменениям

Ожидается, что вклад:

- проходит `cargo test`
- не ломает Python bindings
- не ломает Node bindings и `npm run test:node`
- не ухудшает поддержку `*.ktalk.ru`
- не добавляет лишней привязки к `centraluniversity.ktalk.ru`

Если изменение касается Chrome extension, желательно:

- сохранить ограничение только на `ktalk.ru` и `*.ktalk.ru`
- не превращать расширение в инструмент широкого назначения
- не ухудшать читаемость экспортируемых логов

## Полезные команды

```bash
cargo test
maturin develop
.venv/bin/python -m pytest pytests
npm run build:native
npm run test:node
```

## Node bindings

Node / TypeScript API живет в `src/interface/node.rs` и gated через cargo feature `node`.

Правила для изменений:

- не переносить сетевую или parser-логику в JS-слой
- новые экспорты сначала добавлять в Rust core / application layer, затем тонко пробрасывать в `node.rs`
- сохранять логическое соответствие Python и Node public API
- для Node-методов предпочитать `Promise`-friendly surface

Локальная сборка Node addon:

```bash
cargo build --release --no-default-features --features node
node ./scripts/copy-node-artifact.mjs --destination npm/native/ktalk_bot.node
```

Для Chrome extension:

- загрузить unpacked extension из `tools/ktalk-packet-inspector`
- воспроизвести сценарий на `*.ktalk.ru`
- сохранить trace
- использовать trace как основание для изменений

## Ближайшие конкретные цели

В рамках текущей фазы проекту особенно нужны:

1. Подтвержденный протокол отправки сообщений в чат
2. Улучшение входящих событий чата и room state
3. Более точная participant timeline
4. Более сильные integration tests на основе capture-логов
5. Улучшение reconnect/disconnect поведения

## Дальше по roadmap

После стабилизации текущей фазы проект может двигаться в сторону:

- полноценного event-driven headless client
- richer room analytics
- media/session stats
- и только затем в сторону настоящего media publishing
