# AGENTS.md

Этот документ нужен как быстрый старт для инженера или агентной системы, которая впервые входит в проект `ktalk-bot`.

## 1. Что это за проект

`ktalk-bot` — headless-клиент для KTalk / Jitsi-подобной конференц-платформы.

Текущее назначение:
- подключаться к комнатам без браузерного UI
- работать через cookies и внутренний engine
- читать историю конференций
- обновлять cookies и проверять авторизацию
- присоединяться к комнате на ограниченное время
- наблюдать участников
- отправлять сообщения в чат

Это не полноценный WebRTC-клиент. Сейчас проект находится на стадии устойчивого headless control client, а не media-capable client.

## 2. Технологический стек

- Rust — основная реализация
- PyO3 — Python bindings
- maturin — сборка / публикация Python wheel
- reqwest — HTTP
- tokio / tokio-tungstenite — async websocket/XMPP/system ws
- pytest — Python smoke/API tests

## 3. Главный принцип проекта

Source of truth для сетевого протокола:
- реальные browser captures
- packet logs
- HAR / websocket frame captures

Не полагаться на догадки, если есть подтвержденный трафик.

## 4. Текущий публичный Python API

Основной entrypoint:

```python
import ktalk_bot

client = ktalk_bot.create_engine(
    "sessionToken=...; ngtoken=...; kontur_ngtoken=..."
)
```

Или напрямую:

```python
client = ktalk_bot.KTalkClient(
    "sessionToken=...; ngtoken=...; kontur_ngtoken=..."
)
```

Текущие методы:
- `renew_cookies()`
- `get_history(max_pages=10, page_size=25)`
- `join_room(link, duration_seconds=15)`
- `record_participants(link, duration_seconds=15)`
- `send_chat_message(link, text)`
- `play_audio_on_mic(...)`

Важно:
- `play_audio_on_mic(...)` сейчас намеренно не реализован как реальный media publishing и должен бросать `NotImplementedError`

## 5. Архитектура crate

Основные слои:

- `src/domain/`
  - value objects и доменные типы
  - `auth.rs`
  - `room.rs`
  - `history.rs`
  - `bot.rs`

- `src/application/`
  - orchestration / use cases
  - `bot_engine.rs`
  - `history.rs`

- `src/infrastructure/`
  - HTTP клиенты, DTO, parsing
  - `http/`
  - `parsing/`

- `src/interface/`
  - Python bindings
  - `python.rs`

Реэкспорт наружу идет через:
- `src/lib.rs`

## 6. Важные файлы

Ключевая логика:
- `src/application/bot_engine.rs`
- `src/infrastructure/http/ktalk_http_client.rs`
- `src/interface/python.rs`

Парсинг:
- `src/infrastructure/parsing/history_mapper.rs`
- `src/infrastructure/parsing/xmpp.rs`

Доменные ограничения:
- `src/domain/room.rs`
- `src/domain/auth.rs`
- `src/error.rs`

Документация:
- `README.md`
- `CONTRIBUTING.md`

Тесты:
- `tests/history_flow.rs`
- `pytests/test_public_api.py`

## 7. Поддерживаемые домены

Проект должен поддерживать:
- `ktalk.ru`
- любые `*.ktalk.ru`

Пример:
- `centraluniversity.ktalk.ru`
- любые другие поддомены KTalk

Важно:
- нельзя снова зашивать логику в один конкретный хост
- per-room / per-link host resolution уже поддерживается и не должна ломаться

Для локального тестирования разрешены loopback hosts:
- `localhost`
- `127.0.0.1`
- `::1`

## 8. Что уже известно про протокол

### HTTP

Используются endpoints вида:
- `/api/context`
- `/api/rooms/{short_name}`
- `/api/UserActivities`
- `/api/conferenceshistory`

Cookies являются основным входом в engine.
Минимально важный cookie:
- `sessionToken`

Часто также нужны:
- `ngtoken`
- `kontur_ngtoken`

### System WebSocket

Наблюденные действия:
- `connect`
- `message_subscribe`
- `user_status`
- `chat_join`
- `resource_active_get`
- `ping`
- `chat_ping`

Важно:
- outbound `chat_send` пока не подтвержден capture-логом, это все еще предположение
- если будет новый packet log с реальной отправкой сообщения, эту часть нужно привести к точному parity

### XMPP / Jitsi side

Текущий flow ближе к browser trace, чем к исходному Python shitcode.

Наблюденная последовательность:
- `<open>`
- `<auth mechanism="ANONYMOUS">`
- `<open>`
- `<bind>`
- `<session>`
- focus `conference` IQ
- `presence`

Важно:
- current implementation intentionally follows observed packet log, not only the old scripts
- полноценный Jingle / session-accept / media publishing пока не реализован

## 9. Что не завершено

Критичные незавершенные зоны:
- точный outbound chat-send protocol
- richer incoming chat/events support
- reconnect/disconnect lifecycle
- room state/event stream
- полноценная media publishing path

Самая тяжелая незавершенная часть:
- реальный WebRTC media publishing (audio injection, ICE/DTLS/SRTP, Jingle parity)

Это отдельная большая фаза проекта, не надо смешивать ее с текущим stabilization work.

## 10. Reverse engineering tooling

В репозитории есть Chrome extension:
- `tools/ktalk-packet-inspector`

Назначение:
- захват HTTP events
- захват websocket handshakes
- захват websocket frame sent/received
- экспорт JSON для последующего анализа
- копирование нужных cookies в clipboard

Ограничение:
- работает только на `ktalk.ru` и `*.ktalk.ru`

Если исследуется новый протокол или новый endpoint:
1. использовать extension
2. сохранить packet log
3. использовать лог как основание для правки Rust-кода

## 11. Текущая фаза проекта

Проект в фазе:
- stabilization of headless session layer

Главные цели сейчас:
- довести HTTP + system websocket + XMPP до высокой точности browser parity
- убрать guessed behavior, где есть захваты
- усилить тесты на основе реальных packet logs
- улучшить reliability и lifecycle

## 12. Что делать в первую очередь при новых задачах

Если приходит новая feature-задача:

1. Проверить, есть ли packet log / HAR / capture
2. Проверить, уже есть ли поддержка в `bot_engine.rs`
3. Проверить, не предполагается ли уже что-то без подтвержденного traffic
4. Внести минимальное изменение, которое приближает parity
5. Добавить тест / fixture

Если приходит задача на media:
- сначала уточнить, это passive observation или real media publishing
- если real media publishing, считать это отдельным уровнем сложности

## 13. Чего делать не надо

Не стоит без отдельного обоснования:
- снова привязывать код к `centraluniversity.ktalk.ru`
- раздувать Python API до сложного async surface
- смешивать transport/parser/domain responsibilities
- делать большой рефакторинг без конкретного протокольного выигрыша
- утверждать протокол без capture-данных

## 14. Команды разработки

Основные:

```bash
cargo test
maturin develop
.venv/bin/python -m pytest pytests
```

Для публикации:

```bash
scripts/release.sh build
scripts/release.sh publish
scripts/release.sh release
```

## 15. CI / release

В проекте уже есть GitHub Actions workflow для wheel/sdist release.

Важные факты:
- версия релиза в CI может браться из tag name вида `vX.Y.Z`
- проект публикуется на PyPI как `ktalk-bot`
- Python requirement сейчас `>=3.11`

Если меняется release behavior:
- проверить `.github/workflows/CI.yml`
- проверить `pyproject.toml`
- проверить `Cargo.toml`

## 16. Минимальный mental model для старта

Смотреть на проект надо так:

- `Cookie header` -> engine
- engine -> HTTP bootstrap
- engine -> system websocket + XMPP orchestration
- engine -> Python-facing simple API
- packet captures -> protocol truth

Кратко:
- это headless client first
- не media client yet
- реальные сетевые захваты важнее старых скриптов, если они расходятся
