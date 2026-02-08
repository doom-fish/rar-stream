#!/usr/bin/env node
'use strict'

const { existsSync, readFileSync, createWriteStream, unlinkSync } = require('fs')
const { join } = require('path')
const https = require('https')
const http = require('http')

const REPO = 'doom-fish/rar-stream'
const PKG_DIR = join(__dirname, '..')

function isMusl() {
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch {
      return true
    }
  }
  const { glibcVersionRuntime } = process.report.getReport().header
  return !glibcVersionRuntime
}

function getBinaryName() {
  const { platform, arch } = process
  switch (platform) {
    case 'win32':
      if (arch === 'x64') return 'rar-stream.win32-x64-msvc.node'
      break
    case 'darwin':
      if (arch === 'x64') return 'rar-stream.darwin-x64.node'
      if (arch === 'arm64') return 'rar-stream.darwin-arm64.node'
      break
    case 'linux':
      if (arch === 'x64') return isMusl() ? 'rar-stream.linux-x64-musl.node' : 'rar-stream.linux-x64-gnu.node'
      if (arch === 'arm64') return isMusl() ? 'rar-stream.linux-arm64-musl.node' : 'rar-stream.linux-arm64-gnu.node'
      break
  }
  return null
}

function download(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith('https') ? https : http
    client.get(url, { headers: { 'User-Agent': 'rar-stream-postinstall' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return download(res.headers.location).then(resolve, reject)
      }
      if (res.statusCode !== 200) {
        res.resume()
        return reject(new Error(`HTTP ${res.statusCode} for ${url}`))
      }
      resolve(res)
    }).on('error', reject)
  })
}

async function main() {
  const binaryName = getBinaryName()
  if (!binaryName) {
    console.log(`rar-stream: no prebuilt binary for ${process.platform}-${process.arch}, skipping download`)
    return
  }

  const dest = join(PKG_DIR, binaryName)
  if (existsSync(dest)) return // already present (local build or cached)

  const version = require(join(PKG_DIR, 'package.json')).version
  const url = `https://github.com/${REPO}/releases/download/v${version}/${binaryName}`

  console.log(`rar-stream: downloading ${binaryName} from GitHub Release v${version}...`)

  try {
    const res = await download(url)
    const file = createWriteStream(dest)
    await new Promise((resolve, reject) => {
      res.pipe(file)
      file.on('finish', resolve)
      file.on('error', (err) => { unlinkSync(dest); reject(err) })
    })
    console.log(`rar-stream: installed ${binaryName}`)
  } catch (err) {
    console.warn(`rar-stream: failed to download binary (${err.message}). You may need to build from source.`)
  }
}

main()
