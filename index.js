'use strict'

const fs = require('fs')
const path = require('path')

function resolveBindingPath() {
  const target = `${process.platform}-${process.arch}`
  const candidates = [
    path.join(__dirname, 'npm', 'platforms', target, 'ktalk_bot.node'),
    path.join(__dirname, 'npm', 'native', 'ktalk_bot.node')
  ]

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }

  throw new Error(
    `No prebuilt ktalk-bot native binding found for ${target}. ` +
      'Install a package that includes this platform, or rebuild from source with `npm run build:native`.'
  )
}

const native = require(resolveBindingPath())

class KTalkClient {
  constructor(cookie_header, base_url = null, room_link = null, session_token = null) {
    this._native = new native.KTalkClient(cookie_header, base_url, room_link, session_token)
  }

  bind_room(link) {
    return this._native.bindRoom(link)
  }

  current_room() {
    return this._native.currentRoom()
  }

  renew_cookies() {
    return this._native.renewCookies()
  }

  get_history(max_pages = null, page_size = null) {
    return this._native.getHistory(max_pages, page_size)
  }

  join_room(link = null, duration_seconds = null) {
    return this._native.joinRoom(link, duration_seconds)
  }

  record_participants(link = null, duration_seconds = null) {
    return this._native.recordParticipants(link, duration_seconds)
  }

  send_chat_message(text, link = null) {
    return this._native.sendChatMessage(text, link)
  }

  play_audio_on_mic(audio_path, duration_seconds = null, link = null) {
    return this._native.playAudioOnMic(audio_path, duration_seconds, link)
  }
}

function create_engine(cookie_header, base_url = null, room_link = null, session_token = null) {
  native.createEngine(cookie_header, base_url, room_link, session_token)
  return new KTalkClient(cookie_header, base_url, room_link, session_token)
}

module.exports = {
  create_engine,
  KTalkClient
}
