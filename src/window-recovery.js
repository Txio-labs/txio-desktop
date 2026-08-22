export function formatRendererCrashSummary(details = {}) {
    const reason = details.reason ?? "unknown"
    const exitCode = details.exitCode ?? "unknown"
    const name = details.name ?? "renderer"
    const isMainFrame = details.isMainFrame ?? "unknown"
    const killed = details.killed === true ? "killed=true" : "killed=false"

    return `render-process-gone: ${name} reason=${reason}, exitCode=${exitCode}, ${killed}, isMainFrame=${isMainFrame}`
}

export function shouldReloadAfterCrash(lastCrashAt, now = Date.now(), cooldownMs = 15000) {
    if (!Number.isFinite(lastCrashAt) || lastCrashAt <= 0) {
        return true
    }

    return now - lastCrashAt >= cooldownMs
}
