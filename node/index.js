'use strict'

const { existsSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

// Maps Node.js platform/arch to the target triple used by napi-rs CLI.
function getLocalPath() {
  const platformMap = {
    win32: {
      x64: 'epub-gen-rs.win32-x64-msvc.node',
      arm64: 'epub-gen-rs.win32-arm64-msvc.node',
    },
    linux: {
      x64: 'epub-gen-rs.linux-x64-gnu.node',
      arm64: 'epub-gen-rs.linux-arm64-gnu.node',
    },
    darwin: {
      x64: 'epub-gen-rs.darwin-x64.node',
      arm64: 'epub-gen-rs.darwin-arm64.node',
    },
  }

  const name = platformMap[platform] && platformMap[platform][arch]
  if (!name) {
    throw new Error(`Unsupported platform/arch: ${platform}/${arch}`)
  }
  return join(__dirname, name)
}

let binding

const localPath = getLocalPath()
if (existsSync(localPath)) {
  binding = require(localPath)
} else {
  // Fallback: try the generic name produced by `napi build` without --platform
  const genericPath = join(__dirname, 'epub-gen-rs.node')
  if (existsSync(genericPath)) {
    binding = require(genericPath)
  } else {
    throw new Error(
      `Native binding not found at ${localPath}. ` +
      'Run `npm run build` inside the node/ directory first.'
    )
  }
}

module.exports = binding
