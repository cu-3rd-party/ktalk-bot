# `ktalk-bot`

`ktalk-bot` — это Python-библиотека с реализацией на Rust для работы с KTalk(Jitsi-подобным клиентом конференций).

Проект использует:
- `PyO3` для Python API
- `maturin` для сборки и публикации
- Rust-код для сетевой логики, парсинга и внутренних engine-объектов

## Что умеет библиотека

На текущий момент библиотека предоставляет:
- создание внутреннего engine из строки `Cookie`
- обновление cookies и проверку авторизации
- чтение истории конференций
- подключение к комнате на ограниченное время
- запись замеченных участников во время сессии
- отправку сообщений в чат комнаты

Текущее ограничение:
- полноценная публикация аудио в микрофонный канал пока не реализована

## Roadmap

Ниже приведен практический roadmap развития `ktalk-bot` как headless-клиента для KTalk / Jitsi-подобной платформы.

### Фаза 1. Стабилизация headless-сессии

Цель:
- сделать текущее подключение к комнате, авторизацию и сетевую оркестрацию предсказуемыми и тестируемыми

Задачи:
- довести до высокой точности HTTP + system websocket + XMPP parity
- подтвердить реальный протокол отправки сообщений в чат по захваченным пакетам
- добавить корректный lifecycle: `join`, `keepalive`, `leave`, `disconnect`
- добавить retry/reconnect стратегию
- улучшить диагностические сообщения и wire-level логирование
- расширить fixture-based тесты по захваченным трафикам

### Фаза 2. Headless automation client

Цель:
- превратить библиотеку в пригодный для автоматизации headless-клиент

Задачи:
- получение входящих сообщений чата
- поток событий из engine: `joined`, `participant_seen`, `chat_message`, `disconnected`, `error`
- снимки состояния комнаты
- работа с несколькими комнатами
- CLI-обертка поверх библиотеки

### Фаза 3. Инструменты реверс-инжиниринга

Цель:
- ускорить исследование протокола и поддержку новых инстансов KTalk

Задачи:
- встроенный экспорт нормализованных trace-событий
- diff между браузерным trace и trace из headless engine
- обнаружение capability / совместимости разных `*.ktalk.ru`
- накопление библиотеки fixture-захватов

### Фаза 4. Функции взаимодействия с комнатой

Цель:
- расширить возможности headless-клиента beyond simple join

Задачи:
- история чата и входящие сообщения
- аналитика присутствия
- таймлайн участников
- данные о комнате и ресурсах
- дополнительные room-level API, если они доступны на сервере

### Фаза 5. Media-adjacent features

Цель:
- собирать и анализировать медиа-связанные данные без полной публикации потока

Задачи:
- media/session stats
- Jingle/session metadata
- active speaker / participant activity heuristics
- сетевые метрики качества

### Фаза 6. Полноценная публикация медиа

Цель:
- превратить `ktalk-bot` из headless control client в media-capable client

Задачи:
- полноценный Jingle flow
- ICE/DTLS/SRTP
- публикация Opus audio
- инъекция аудио в микрофонный канал
- возможно, в будущем, video/screen-share

Это самая сложная часть roadmap и должна разрабатываться как отдельный слой поверх уже стабильного headless-клиента.

### Рекомендуемый порядок реализации

1. Надежность текущего протокольного слоя
2. Входящие события и входящий чат
3. Состояние комнаты и аналитика участников
4. Trace / reverse-engineering tooling
5. Media stats
6. Полноценный media publishing

## Установка

Для разработки:

```bash
maturin develop
```

Для обычной сборки wheel:

```bash
maturin build --release
```

## Базовая идея API

Публичный Python API построен вокруг black-box engine:

```python
import ktalk_bot

client = ktalk_bot.create_engine(
    "sessionToken=...; ngtoken=...; kontur_ngtoken=..."
)
```

Можно создать объект и напрямую:

```python
import ktalk_bot

client = ktalk_bot.KTalkClient(
    "sessionToken=...; ngtoken=...; kontur_ngtoken=..."
)
```

## Примеры использования

### Обновить cookies и получить профиль

```python
profile = client.renew_cookies()
print(profile["user_id"])
print(profile["first_name"], profile["last_name"])
```

### Получить историю конференций

```python
history = client.get_history(max_pages=2, page_size=25)

for item in history:
    print(item["title"], item["room_name"])
```

### Подключиться к комнате

```python
report = client.join_room(
    "https://centraluniversity.ktalk.ru/ewwv291ipuud",
    duration_seconds=15,
)

print(report["joined"])
print(report["participants"])
```

### Записать замеченных участников

```python
participants = client.record_participants(
    "https://centraluniversity.ktalk.ru/ewwv291ipuud",
    duration_seconds=15,
)
```

### Отправить сообщение в чат

```python
client.send_chat_message(
    "https://centraluniversity.ktalk.ru/ewwv291ipuud",
    "Привет из ktalk-bot",
)
```

## Формат данных

### `renew_cookies()`

Возвращает:

```python
{
    "user_id": str,
    "first_name": str,
    "last_name": str,
}
```

### `get_history()`

Возвращает список словарей:

```python
[
    {
        "key": str | None,
        "room_name": str,
        "title": str,
        "start_time": str | None,
        "end_time": str | None,
        "participants_count": int,
        "participants": list[str],
        "participant_details": list[dict[str, str]],
        "has_recording": bool,
        "recording_id": str | None,
        "recording_url": str | None,
        "recording": dict[str, str] | None,
    }
]
```

### `join_room()`

Возвращает:

```python
{
    "room_name": str,
    "conference_id": str,
    "joined": bool,
    "participants": [
        {
            "occupant_id": str,
            "display_name": str,
            "user_id": str | None,
        }
    ],
}
```

## Документация из Python

Python-методы снабжены встроенной документацией и сигнатурами. Например:

```python
import ktalk_bot

help(ktalk_bot.create_engine)
help(ktalk_bot.KTalkClient)
help(ktalk_bot.KTalkClient.get_history)
```

Ожидаемая сигнатура для функции создания клиента:

```python
create_engine(cookie_header: str, base_url: str = "https://centraluniversity.ktalk.ru") -> KTalkClient
```

## Разработка

Полезные команды:

```bash
cargo test
maturin develop
.venv/bin/python -m pytest pytests
```

## Публикация

В репозитории есть CI для сборки wheel/sdist и публикации на PyPI.

Также доступен локальный сценарий:

```bash
scripts/release.sh build
scripts/release.sh publish
scripts/release.sh release
```

## Contributing

Подробные правила и текущие цели разработки описаны в [`CONTRIBUTING.md`](./CONTRIBUTING.md).
