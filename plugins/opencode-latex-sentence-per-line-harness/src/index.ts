import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs"
import { isAbsolute, relative, resolve, sep } from "node:path"
import type { Plugin } from "@opencode-ai/plugin"

const TEX_EDIT_TOOL_RE = /(?:^|[._-])(edit|write|patch|apply_patch)(?:$|[._-])/i
const PATCH_HEADER_RE = /^\*\*\*\s+(?:Add File|Update File):\s+(.+?)\s*$/gm
const SENTENCE_BOUNDARY_RE = /([.!?])\s+(?=[A-Z\\])/g
const IGNORED_PATH_SEGMENTS = new Set([
  ".git",
  ".hg",
  ".svn",
  "node_modules",
  "vendor",
  "dist",
  "build",
  "target",
  "_build",
  "__pycache__",
])

type FormatResult = {
  path: string
  changed: boolean
  skipped?: string
  error?: string
}

function splitCodeAndComment(line: string): [string, string] {
  let escaped = false
  for (let index = 0; index < line.length; index++) {
    const char = line[index]
    if (char === "\\" && !escaped) {
      escaped = true
      continue
    }
    if (char === "%" && !escaped) return [line.slice(0, index), line.slice(index)]
    escaped = false
  }
  return [line, ""]
}

function splitSentences(text: string): string {
  return text
    .split(/(?<=\n)/)
    .map((line) => {
      const hasNewline = line.endsWith("\n")
      const lineBody = hasNewline ? line.slice(0, -1) : line
      const [code, comment] = splitCodeAndComment(lineBody)
      return code.replace(SENTENCE_BOUNDARY_RE, (_match, punctuation: string) => `${punctuation}\n`) + comment + (hasNewline ? "\n" : "")
    })
    .join("")
}

function isSafeRelativePath(path: string): boolean {
  const normalized = path.split(/[\\/]+/)
  if (normalized.some((part) => part === ".." || part === "" || IGNORED_PATH_SEGMENTS.has(part))) return false
  return true
}

function isInside(root: string, path: string): boolean {
  const rel = relative(root, path)
  return rel === "" || (!rel.startsWith("..") && !isAbsolute(rel))
}

function resolveCandidatePath(path: string, roots: string[]): string | undefined {
  if (!path || path.includes("\0")) return undefined
  const trimmed = path.trim().replace(/^['"]|['"]$/g, "")
  if (!trimmed.endsWith(".tex")) return undefined

  if (isAbsolute(trimmed)) {
    const absolute = resolve(trimmed)
    return roots.some((root) => isInside(root, absolute)) ? absolute : undefined
  }

  if (!isSafeRelativePath(trimmed)) return undefined
  for (const root of roots) {
    const absolute = resolve(root, trimmed)
    if (isInside(root, absolute)) return absolute
  }
  return undefined
}

function collectStringPaths(value: unknown, paths: Set<string>, depth = 0): void {
  if (depth > 5 || value == null) return
  if (typeof value === "string") {
    if (value.endsWith(".tex")) paths.add(value)
    return
  }
  if (Array.isArray(value)) {
    for (const item of value) collectStringPaths(item, paths, depth + 1)
    return
  }
  if (typeof value === "object") {
    for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
      if (/^(file|filepath|file_path|path|oldpath|newpath|old_path|new_path)$/i.test(key) && typeof item === "string") {
        paths.add(item)
      }
      collectStringPaths(item, paths, depth + 1)
    }
  }
}

function collectPatchPaths(patchText: unknown, paths: Set<string>): void {
  if (typeof patchText !== "string") return
  for (const match of patchText.matchAll(PATCH_HEADER_RE)) paths.add(match[1])
}

function findTouchedTexFiles(args: unknown, roots: string[]): string[] {
  const raw = new Set<string>()
  collectStringPaths(args, raw)
  collectPatchPaths((args as Record<string, unknown> | undefined)?.patchText, raw)
  collectPatchPaths((args as Record<string, unknown> | undefined)?.patch, raw)

  const resolved = new Set<string>()
  for (const path of raw) {
    const candidate = resolveCandidatePath(path, roots)
    if (candidate) resolved.add(candidate)
  }
  return [...resolved]
}

function hasPathSegment(path: string, segment: string): boolean {
  return path.split(sep).includes(segment)
}

function shouldSkipExistingFile(path: string): string | undefined {
  if (!existsSync(path)) return "file does not exist after edit"
  try {
    if (!statSync(path).isFile()) return "not a regular file"
  } catch (error) {
    return error instanceof Error ? error.message : String(error)
  }
  for (const segment of IGNORED_PATH_SEGMENTS) {
    if (hasPathSegment(path, segment)) return `ignored path segment: ${segment}`
  }
  return undefined
}

function formatTexFile(path: string): FormatResult {
  const skipped = shouldSkipExistingFile(path)
  if (skipped) return { path, changed: false, skipped }

  try {
    const original = readFileSync(path, "utf8")
    const updated = splitSentences(original)
    if (updated === original) return { path, changed: false }
    writeFileSync(path, updated, "utf8")
    return { path, changed: true }
  } catch (error) {
    return { path, changed: false, error: error instanceof Error ? error.message : String(error) }
  }
}

function relativeToAnyRoot(path: string, roots: string[]): string {
  for (const root of roots) {
    if (isInside(root, path)) return relative(root, path) || path
  }
  return path
}

function harnessInstructions(): string {
  return [
    "## LaTeX sentence-per-line harness",
    "A local opencode plugin automatically rewrites touched `.tex` files after Edit/Write/Patch tools so prose keeps one sentence per line.",
    "The rewrite preserves LaTeX comments beginning with `%` and only inserts newlines after `.`, `!`, or `?` followed by an uppercase token or LaTeX command.",
    "After editing prose-heavy `.tex` files, still review the diff and manually repair awkward splits around abbreviations, citations, math-adjacent prose, commands, or tables.",
    "Do not rely on the harness for `.bib`, `.sty`, `.cls`, generated files, or non-prose TeX fragments.",
  ].join("\n")
}

export default (async ({ directory, worktree }) => {
  const roots = [...new Set([worktree, directory].filter((item): item is string => typeof item === "string" && item.length > 0).map((item) => resolve(item)))]

  return {
    "experimental.chat.system.transform": async (_input, output) => {
      output.system.push(harnessInstructions())
    },

    "tool.execute.after": async (input, output) => {
      if (!TEX_EDIT_TOOL_RE.test(input.tool)) return

      const files = findTouchedTexFiles(input.args, roots)
      if (files.length === 0) return

      const results = files.map(formatTexFile)
      const changed = results.filter((result) => result.changed)
      const errors = results.filter((result) => result.error)

      if (changed.length > 0) {
        output.output += `\n\n[latex-sentence-per-line] formatted: ${changed.map((result) => relativeToAnyRoot(result.path, roots)).join(", ")}`
      }
      if (errors.length > 0) {
        output.output += `\n\n[latex-sentence-per-line] errors: ${errors.map((result) => `${relativeToAnyRoot(result.path, roots)} (${result.error})`).join(", ")}`
      }
    },
  }
}) satisfies Plugin
