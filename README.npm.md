# ktalk-bot for Node.js

`ktalk-bot` exposes the Rust headless KTalk engine to Node.js through a native N-API addon.

## Install

```bash
npm install ktalk-bot
```

The published package includes prebuilt binaries for:

- `linux-x64`
- `darwin-x64`
- `win32-x64`

If your platform is not packaged, installation falls back to a local Rust build. That requires `cargo` and a working Rust toolchain.

## TypeScript usage

```ts
import { create_engine, KTalkClient } from 'ktalk-bot'

const client = create_engine('ngtoken=...; kontur_ngtoken=...')

const history = await client.get_history(2, 25)
console.log(history[0]?.room_name)
```

## API

- `create_engine(cookie_header, base_url?, room_link?, session_token?)`
- `new KTalkClient(cookie_header, base_url?, room_link?, session_token?)`
- `await client.renew_cookies()`
- `await client.get_history(max_pages?, page_size?)`
- `await client.join_room(link?, duration_seconds?)`
- `await client.record_participants(link?, duration_seconds?)`
- `await client.send_chat_message(text, link?)`
- `await client.play_audio_on_mic(audio_path, duration_seconds?, link?)`

`play_audio_on_mic` remains intentionally unimplemented until real media publishing support lands in the Rust core.
