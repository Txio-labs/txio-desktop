import path from "path"
import { fileURLToPath, pathToFileURL } from "url"

export function isPathInside(childPath, parentPath) {
  const relative = path.relative(parentPath, childPath)
  return !relative.startsWith("..") && !path.isAbsolute(relative)
}

export function isUrlInsideRoot(targetUrl, rootUrl) {
  try {
    const target = new URL(targetUrl)
    const root = new URL(rootUrl)
    if (target.protocol !== root.protocol) return false
    if (target.protocol === "file:") {
      const targetPath = path.resolve(fileURLToPath(target.href))
      const rootPath = path.resolve(fileURLToPath(root.href))
      return targetPath === rootPath || isPathInside(targetPath, rootPath)
    }
    return (
      target.origin === root.origin &&
      (target.pathname === root.pathname ||
        target.pathname.startsWith(root.pathname.endsWith("/") ? root.pathname : root.pathname + "/"))
    )
  } catch {
    return false
  }
}
