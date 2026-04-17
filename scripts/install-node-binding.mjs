import fs from 'fs'
import path from 'path'
import { spawnSync } from 'child_process'

const platformBinary = path.resolve(
  'npm',
  'platforms',
  `${process.platform}-${process.arch}`,
  'ktalk_bot.node'
)

if (fs.existsSync(platformBinary)) {
  process.exit(0)
}

if (process.env.KTALK_BOT_SKIP_INSTALL === '1') {
  process.exit(0)
}

const cargoCheck = spawnSync('cargo', ['--version'], { stdio: 'ignore' })
if (cargoCheck.status !== 0) {
  console.warn(
    `ktalk-bot: no packaged native binding for ${process.platform}-${process.arch}, and cargo is unavailable for source build fallback`
  )
  process.exit(0)
}

const build = spawnSync(
  'cargo',
  ['build', '--release', '--no-default-features', '--features', 'node'],
  { stdio: 'inherit' }
)

if (build.status !== 0) {
  process.exit(build.status ?? 1)
}

const copy = spawnSync('node', ['./scripts/copy-node-artifact.mjs', '--destination', 'npm/native/ktalk_bot.node'], {
  stdio: 'inherit'
})

process.exit(copy.status ?? 1)
