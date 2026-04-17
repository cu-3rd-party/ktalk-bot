# KTalk Packet Inspector

Локальное Chrome-расширение для отладки и исследования сетевого протокола KTalk.

Расширение:
- работает только на доменах `ktalk.ru` и `*.ktalk.ru`
- умеет захватывать HTTP-события через `webRequest`
- умеет захватывать WebSocket handshake и WebSocket frames через Chrome Debugger Protocol
- сохраняет журнал локально в `chrome.storage.local`
- позволяет открыть отдельную страницу с фильтрацией и экспортом JSON

## Как установить

1. Откройте `chrome://extensions`
2. Включите `Режим разработчика`
3. Нажмите `Load unpacked`
4. Выберите каталог:

```text
tools/ktalk-packet-inspector
```

## Как использовать

1. Откройте любую страницу на `*.ktalk.ru`
2. Нажмите на иконку расширения
3. Нажмите `Подключить захват`
4. Выполните нужные действия в KTalk
5. Нажмите `Открыть журнал`
6. Отфильтруйте события или экспортируйте JSON

## Что именно попадает в журнал

- HTTP requests
- request headers
- response headers
- WebSocket creation
- WebSocket handshake
- WebSocket frame sent
- WebSocket frame received
- WebSocket close

## Ограничения

- расширение рассчитано на инженерную отладку, а не на фоновый массовый сбор
- Debugger API подключается к конкретной вкладке
- захват бинарных данных сохраняется в том виде, в каком их отдает Chrome Debugger API


## Извлечение авторизации

Кнопка копирования берёт:
- `ngtoken`
- `kontur_ngtoken`
- `session token` из `localStorage["session"]`, если он найден

В буфер обмена копируется строка cookies и отдельная строка `session_token=...`.
