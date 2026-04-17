import { create_engine, KTalkClient } from '../../index'

async function main() {
  const client: KTalkClient = create_engine('ngtoken=test; kontur_ngtoken=test')
  const history = await client.get_history(1, 1)
  const roomName: string | undefined = history[0]?.room_name ?? undefined
  console.log(roomName)
}

void main()
