'use strict'

const assert = require('assert')
const { create_engine, KTalkClient } = require('../../index.js')

async function main() {
  assert.strictEqual(typeof create_engine, 'function')
  assert.strictEqual(typeof KTalkClient, 'function')

  const client = create_engine('ngtoken=test; kontur_ngtoken=test', 'https://centraluniversity.ktalk.ru')
  assert.ok(client instanceof KTalkClient)

  let reachedBinding = false
  try {
    const history = await client.get_history(1, 1)
    reachedBinding = true
    assert.ok(Array.isArray(history))
  } catch (error) {
    reachedBinding = true
    assert.ok(error instanceof Error)
    assert.ok(error.message.length > 0)
  }

  assert.ok(reachedBinding, 'get_history should reach the native binding')
}

main().catch((error) => {
  console.error(error)
  process.exit(1)
})
