import fs from 'fs'
import path from 'path'

function parseArgs(argv) {
  const args = new Map()
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index]
    const value = argv[index + 1]
    if (key.startsWith('--') && value) {
      args.set(key, value)
      index += 1
    }
  }
  return args
}

function sourceCandidates() {
  const releaseDir = path.resolve('target', 'release')
  return [
    path.join(releaseDir, 'ktalk_bot.node'),
    path.join(releaseDir, 'libktalk_bot.so'),
    path.join(releaseDir, 'libktalk_bot.dylib'),
    path.join(releaseDir, 'ktalk_bot.dll')
  ]
}

const args = parseArgs(process.argv.slice(2))
const destination = args.get('--destination')

if (!destination) {
  throw new Error('missing --destination for copy-node-artifact script')
}

const source = sourceCandidates().find((candidate) => fs.existsSync(candidate))
if (!source) {
  throw new Error(
    'native artifact was not found in target/release; run `cargo build --release --no-default-features --features node` first'
  )
}

const destinationPath = path.resolve(destination)
fs.mkdirSync(path.dirname(destinationPath), { recursive: true })
fs.copyFileSync(source, destinationPath)
