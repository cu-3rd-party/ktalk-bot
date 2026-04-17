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
