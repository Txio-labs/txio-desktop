import { app, BrowserWindow, shell, session, protocol } from "electron"
import path from "path"
import { fileURLToPath, pathToFileURL } from "url"
import crypto from "crypto"
import fs from "fs"
import { isPathInside, isUrlInsideRoot } from "./security.js"

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

// Scope navigation and file access to the app's own packaged directory
const APP_ROOT_PATH = path.resolve(__dirname, "..")
const APP_ROOT_URL = pathToFileURL(APP_ROOT_PATH).href
const DIST_PATH = path.resolve(__dirname, "../dist")

// Global safety net for unhandled promise rejections
process.on("unhandledRejection", (err) => {
  console.error("Unhandled promise rejection in main process:", err)
  if (app) app.quit()
})

const CSP_NONCE = crypto.randomBytes(16).toString("base64")

const CSP = [
  "default-src 'self'",
  `script-src 'self' 'nonce-${CSP_NONCE}'`,
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob: https:",
  "font-src 'self' data:",
  "connect-src 'self' https: wss:",
  "frame-src 'self' https:",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'self'",
].join("; ")

const createWindow = () => {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    icon: path.join(__dirname, "assets", "logo-B-VeRwKK.png"),
    webPreferences: {
      sandbox: true,
      contextIsolation: true,
      nodeIntegration: false,
      webSecurity: true,
    },
  })

  win.webContents.setWindowOpenHandler(({ url }) => {
    try {
      const parsed = new URL(url)
      if (parsed.protocol === "http:" || parsed.protocol === "https:") {
        shell.openExternal(url)
      }
    } catch {
      // ignore invalid URLs
    }
    return { action: "deny" }
  })

  win.webContents.on("console-message", (event, level, message, line, sourceId) => {
    if (message.toLowerCase().includes("content security policy") || message.toLowerCase().includes("csp")) {
      console.error(`CSP Violation: ${message}`)
    }
  })

  win.webContents.on("will-navigate", (event, url) => {
    if (!isUrlInsideRoot(url, APP_ROOT_URL)) {
      event.preventDefault()
    }
  })

  win.loadFile(path.join(__dirname, "../dist/index.html")).catch((err) => {
    console.error("Failed to load app HTML:", err)
    app.quit()
  })
}

app
  .whenReady()
  .then(() => {
    protocol.handle("file", async (request) => {
      try {
        const filePath = path.resolve(fileURLToPath(request.url))

        // Restrict file serving to the app root directory (prevent arbitrary file reads)
        if (filePath !== APP_ROOT_PATH && !isPathInside(filePath, APP_ROOT_PATH)) {
          return new Response("Forbidden", { status: 403 })
        }

        const content = await fs.promises.readFile(filePath)

        const ext = path.extname(filePath).toLowerCase()
        let mimeType = "application/octet-stream"
        if (ext === ".html") mimeType = "text/html"
        else if (ext === ".js") mimeType = "text/javascript"
        else if (ext === ".css") mimeType = "text/css"
        else if (ext === ".png") mimeType = "image/png"
        else if (ext === ".svg") mimeType = "image/svg+xml"
        else if (ext === ".json") mimeType = "application/json"
        else if (ext === ".woff2") mimeType = "font/woff2"

        // Only stamp CSP nonce onto HTML files originating from the trusted build directory (dist)
        if (ext === ".html") {
          let html = content.toString("utf-8")
          if (filePath === DIST_PATH || isPathInside(filePath, DIST_PATH)) {
            html = html.replace(/<script(?=[\s>])/g, `<script nonce="${CSP_NONCE}"`)
          }
          return new Response(html, { headers: { "content-type": mimeType } })
        }

        return new Response(content, { headers: { "content-type": mimeType } })
      } catch (err) {
        return new Response("Not Found", { status: 404 })
      }
    })

    // Deny browser permission requests by default for embedded frames
    session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => {
      callback(false)
    })

    if (session.defaultSession.setPermissionCheckHandler) {
      session.defaultSession.setPermissionCheckHandler(() => false)
    }

    session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
      callback({
        responseHeaders: {
          ...details.responseHeaders,
          "Content-Security-Policy": [CSP],
        },
      })
    })

    createWindow()

    app.on("activate", () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        createWindow()
      }
    })
  })
  .catch((err) => {
    console.error("Failed during app initialization:", err)
    app.quit()
  })

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit()
  }
})