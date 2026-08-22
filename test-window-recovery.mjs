import { describe, it } from "node:test"
import assert from "node:assert"
import { formatRendererCrashSummary, shouldReloadAfterCrash } from "./src/window-recovery.js"

describe("Renderer crash recovery", () => {
  it("formats renderer crash details for logging and user messaging", () => {
    const summary = formatRendererCrashSummary({ reason: "crashed", exitCode: 1, killed: false })

    assert.match(summary, /render-process-gone/i)
    assert.match(summary, /crashed/i)
    assert.match(summary, /exitCode=1/i)
  })

  it("prevents rapid reload loops after repeated renderer crashes", () => {
    assert.strictEqual(shouldReloadAfterCrash(1000, 2000, 5000), false)
    assert.strictEqual(shouldReloadAfterCrash(1000, 7000, 5000), true)
  })
})
