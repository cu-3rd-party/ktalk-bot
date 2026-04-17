export interface UserProfile {
  user_id: string
  first_name: string
  last_name: string
}

export interface ParticipantDetail {
  kind: 'authenticated' | 'anonymous'
  display_name: string
}

export interface Recording {
  recording_id: string
  playback_url: string
}

export interface HistoryRecord {
  key: string | null
  room_name: string
  title: string
  start_time: string | null
  end_time: string | null
  participants_count: number
  participants: string[]
  participant_details: ParticipantDetail[]
  has_recording: boolean
  recording_id: string | null
  recording_url: string | null
  recording: Recording | null
}

export interface ParticipantSnapshot {
  occupant_id: string
  display_name: string
  user_id: string | null
}

export interface JoinRoomReport {
  room_name: string
  conference_id: string
  joined: boolean
  participants: ParticipantSnapshot[]
}

export declare class KTalkClient {
  constructor(
    cookie_header: string,
    base_url?: string | null,
    room_link?: string | null,
    session_token?: string | null
  )

  bind_room(link: string): void
  current_room(): string | null
  renew_cookies(): Promise<UserProfile>
  get_history(max_pages?: number | null, page_size?: number | null): Promise<HistoryRecord[]>
  join_room(link?: string | null, duration_seconds?: number | null): Promise<JoinRoomReport>
  record_participants(
    link?: string | null,
    duration_seconds?: number | null
  ): Promise<ParticipantSnapshot[]>
  send_chat_message(text: string, link?: string | null): Promise<void>
  play_audio_on_mic(
    audio_path: string,
    duration_seconds?: number | null,
    link?: string | null
  ): Promise<void>
}

export declare function create_engine(
  cookie_header: string,
  base_url?: string | null,
  room_link?: string | null,
  session_token?: string | null
): KTalkClient
