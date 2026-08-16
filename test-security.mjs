import { describe, it } from "node:test"
import assert from "node:assert"
import path from "path"
import { fileURLToPath, pathToFileURL } from "url"
import { isPathInside, isUrlInsideRoot } from "./src/security.js"

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const APP_ROOT_PATH = path.resolve(__dirname)
const APP_ROOT_URL = pathToFileURL(APP_ROOT_PATH).href

describe("Path & URL Containment Security", () => {
  it("correctly identifies inside paths", () => {
    const child = path.join(APP_ROOT_PATH, "dist", "index.html")
    assert.strictEqual(isPathInside(child, APP_ROOT_PATH), true)
  })

  it("rejects outside paths like /etc/passwd", () => {
    assert.strictEqual(isPathInside("/etc/passwd", APP_ROOT_PATH), false)
  })

  it("rejects sibling directories sharing name prefix", () => {
    const sibling = APP_ROOT_PATH + "-evil-sibling/index.html"
    assert.strictEqual(isPathInside(sibling, APP_ROOT_PATH), false)
  })

  it("validates legitimate in-app URL navigation", () => {
    const legitimateUrl = pathToFileURL(path.join(APP_ROOT_PATH, "dist", "index.html")).href
    assert.strictEqual(isUrlInsideRoot(legitimateUrl, APP_ROOT_URL), true)
  })

  it("rejects sibling directory navigation bypass in will-navigate", () => {
    const siblingUrl = pathToFileURL(APP_ROOT_PATH + "-evil-sibling/malicious.html").href
    assert.strictEqual(isUrlInsideRoot(siblingUrl, APP_ROOT_URL), false)
  })

  it("rejects arbitrary external URLs", () => {
    assert.strictEqual(isUrlInsideRoot("https://evil.com", APP_ROOT_URL), false)
    assert.strictEqual(isUrlInsideRoot("file:///etc/passwd", APP_ROOT_URL), false)
  })
})
