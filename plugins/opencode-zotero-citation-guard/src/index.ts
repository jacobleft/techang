import { existsSync } from "node:fs"
import { join } from "node:path"
import type { Plugin } from "@opencode-ai/plugin"

type SessionState = {
  citationIntent: boolean
  requestedCitekeys: Set<string>
  verifiedCitekeys: Set<string>
  retractionCheckedCitekeys: Set<string>
  itemKeyToCitekey: Map<string, string>
}

type ZoteroCitationGuardOptions = {
  literatureNoteDir?: string
}

const states = new Map<string, SessionState>()

const CITATION_INTENT_RE = /\b(citation|citations|cite|cites|citekey|citekeys|bibliograph(?:y|ic)|reference|references|literature|zotero|better\s*bibtex|bibtex|csl|doi)\b|引用|引文|参考文献|文献|出处|来源|著录|脚注|尾注/i
const CITATION_MARKER_RE = /(?:\[@[-A-Za-z0-9:_]+(?:;\s*@[-A-Za-z0-9:_]+)*\])|(?:@[-A-Za-z][A-Za-z0-9:_-]{2,})|(?:\\(?:cite|parencite|textcite|autocite|footcite|citep|citet)\s*(?:\[[^\]]*\]\s*){0,2}\{[^}]+\})|(?:\b(?:References|Bibliography|Works Cited)\b)|(?:参考文献|引用文献)/i
const CITEKEY_RE = /(?:\[@|(?<![\w-])@)([-A-Za-z][A-Za-z0-9:_-]{2,})|\\(?:cite|parencite|textcite|autocite|footcite|citep|citet)\s*(?:\[[^\]]*\]\s*){0,2}\{([^}]+)\}/g
const ZOTERO_RE = /zotero/i
const PREFERRED_SEARCH_LIBRARY_RE = /^zotero-mcp_search_library$/
const PREFERRED_ITEM_DETAILS_RE = /^zotero-mcp_get_item_details$/
const RETRACTION_CHECK_RE = /^zotero-mcp-54yyyu_scite_enrich_item$|^zotero-mcp-54yyyu_scite_check_retractions$/
const INCOMPLETE_METADATA_RE = /(?:not connected|connection refused|timed out|\berror\b|\bfailed\b|Year:\s*N\/A|Authors?:\s*(?:No authors listed|N\/A|Unknown))/i
const HAS_TITLE_RE = /(?:^|["'\n\r\s{,])(title|Title|publicationTitle|bookTitle)["']?\s*[:=]/
const HAS_AUTHOR_RE = /(?:^|["'\n\r\s{,])(creators?|authors?|Authors?|creator)["']?\s*[:=]/
const HAS_DATE_RE = /(?:^|["'\n\r\s{,])(date|Date|year|Year)["']?\s*[:=]/
const ITEM_KEY_RE = /(?:itemKey|key|Key)["']?\s*[:=]\s*"?([A-Z0-9]{8})"?/g
const BIBLIOGRAPHY_OUTPUT_RE = /(?:^|\n)\s*(?:#{1,6}\s*)?(?:References|Bibliography|Works Cited|参考文献|引用文献)\s*(?:\n|$)|(?:^|\n)\s*\[\d+\]\s+.+\d{4}/i
const DEFAULT_LITERATURE_NOTE_DIR = "/Users/jacob/vaults/lifework/01_notes/0103_literature_notes"

function state(sessionID: string): SessionState {
  let existing = states.get(sessionID)
  if (!existing) {
    existing = {
      citationIntent: false,
      requestedCitekeys: new Set(),
      verifiedCitekeys: new Set(),
      retractionCheckedCitekeys: new Set(),
      itemKeyToCitekey: new Map(),
    }
    states.set(sessionID, existing)
  }
  return existing
}

function textFromParts(parts: any[]): string {
  return parts
    .map((part) => {
      if (typeof part?.text === "string") return part.text
      if (typeof part?.content === "string") return part.content
      return ""
    })
    .join("\n")
}

function extractCitekeys(text: string): string[] {
  const keys = new Set<string>()
  for (const match of text.matchAll(CITEKEY_RE)) {
    const raw = match[1] ?? match[2] ?? ""
    for (const key of raw.split(/[;,]/).map((item) => item.trim()).filter(Boolean)) {
      keys.add(key.replace(/^@/, ""))
    }
  }
  return [...keys]
}

function argsText(args: unknown): string {
  try {
    return JSON.stringify(args ?? {})
  } catch {
    return ""
  }
}

function outputText(output: unknown): string {
  return typeof output === "string" ? output : argsText(output)
}

function hasCompleteMetadata(text: string): boolean {
  if (!text.trim()) return false
  if (INCOMPLETE_METADATA_RE.test(text)) return false
  return HAS_TITLE_RE.test(text) && HAS_AUTHOR_RE.test(text) && HAS_DATE_RE.test(text)
}

function citekeyFromQuery(args: Record<string, unknown>): string | undefined {
  const q = typeof args.q === "string" ? args.q : undefined
  if (!q) return undefined
  const keys = extractCitekeys(q.includes("@") ? q : `@${q}`)
  return keys.length === 1 ? keys[0] : undefined
}

function itemKeyFromArgs(args: Record<string, unknown>): string | undefined {
  const itemKey = args.itemKey ?? args.item_key
  return typeof itemKey === "string" && /^[A-Z0-9]{8}$/.test(itemKey) ? itemKey : undefined
}

function extractItemKeys(text: string): string[] {
  const keys = new Set<string>()
  for (const match of text.matchAll(ITEM_KEY_RE)) keys.add(match[1])
  return [...keys]
}

function recordPreferredSearch(session: SessionState, args: Record<string, unknown>, text: string): void {
  const citekey = citekeyFromQuery(args)
  if (!citekey || !hasCompleteMetadata(text)) return
  session.verifiedCitekeys.add(citekey)
  for (const itemKey of extractItemKeys(text)) session.itemKeyToCitekey.set(itemKey, citekey)
}

function recordPreferredItemDetails(session: SessionState, args: Record<string, unknown>, text: string): void {
  if (!hasCompleteMetadata(text)) return
  const itemKey = itemKeyFromArgs(args)
  if (itemKey) {
    const citekey = session.itemKeyToCitekey.get(itemKey)
    if (citekey) session.verifiedCitekeys.add(citekey)
  }
  for (const citekey of extractCitekeys(text)) session.verifiedCitekeys.add(citekey)
}

function recordRetractionCheck(session: SessionState, args: Record<string, unknown>, text: string): void {
  const itemKey = itemKeyFromArgs(args)
  if (itemKey) {
    const citekey = session.itemKeyToCitekey.get(itemKey)
    if (citekey) session.retractionCheckedCitekeys.add(citekey)
  }
  for (const citekey of extractCitekeys(argsText(args) + "\n" + text)) session.retractionCheckedCitekeys.add(citekey)
  const doiOrCollectionCheck = typeof args.doi === "string" || typeof args.collection === "string" || typeof args.tag === "string"
  if (doiOrCollectionCheck) for (const citekey of session.verifiedCitekeys) session.retractionCheckedCitekeys.add(citekey)
}

function missingLiteratureNotes(citekeys: string[], literatureNoteDir: string): string[] {
  return citekeys.filter((citekey) => !existsSync(join(literatureNoteDir, `${citekey}.md`)))
}

function formatKeys(keys: string[]): string {
  return keys.length > 0 ? keys.map((key) => `\`${key}\``).join(", ") : "none"
}

function citationGuardrail(literatureNoteDir: string): string {
  return [
    "## Zotero citation integrity harness",
    "When working with citations, references, bibliographies, citekeys, DOI metadata, or literature claims:",
    "- Treat citation metadata as untrusted until retrieved from Zotero/Better BibTeX.",
    "- Prefer the `zotero-mcp` server explicitly: first call `zotero-mcp_search_library(q=<citekey>)`, then `zotero-mcp_get_item_details(itemKey=<key>)` when full metadata is needed.",
    "- Do not treat `zotero-mcp-54yyyu_zotero_search_by_citation_key` as sufficient metadata verification when it only returns existence, title, `Year: N/A`, or `Authors: No authors listed`.",
    "- For authors, years, venues, DOI, pages, BibTeX, CSL, or reference-list entries, fetch full Zotero metadata using an item-key metadata tool or a Zotero library search by citekey/title that returns creators/date.",
    `- For citekeys used in drafts, check whether an Obsidian literature note exists under \`${literatureNoteDir}/<citekey>.md\`; if missing, flag it instead of assuming the reading note exists.`,
    "- Before producing a final bibliography/reference list, ensure each cited item has complete Zotero metadata and a retraction/correction risk check (for example via Scite/Zotero retraction tools) or explicitly mark that risk check as not performed.",
    "- Do not invent authors, titles, years, venues, DOIs, page ranges, BibTeX, CSL JSON, or reference-list entries.",
    "- If a citekey is missing from Zotero, say it was not found and ask whether to search/add it; do not fabricate a substitute.",
    "- If Zotero tools are unavailable, explicitly state that citations cannot be verified instead of producing definitive references.",
    "- Prefer preserving citekeys in draft prose (`[@citekey]`) until verified metadata is needed.",
  ].join("\n")
}

export default (async (_input, options?: ZoteroCitationGuardOptions) => {
  const literatureNoteDir = options?.literatureNoteDir ?? process.env.ZOTERO_CITATION_GUARD_LITERATURE_NOTE_DIR ?? DEFAULT_LITERATURE_NOTE_DIR

  return {
    "experimental.chat.system.transform": async (_input, output) => {
      output.system.push(citationGuardrail(literatureNoteDir))
    },

    "chat.message": async (input, output) => {
      const session = state(input.sessionID)
      const text = textFromParts(output.parts)
      for (const key of extractCitekeys(text)) session.requestedCitekeys.add(key)
      if (CITATION_INTENT_RE.test(text) || CITATION_MARKER_RE.test(text)) {
        session.citationIntent = true
      }
    },

    "tool.execute.after": async (input, output) => {
      const session = state(input.sessionID)
      if (!ZOTERO_RE.test(input.tool)) return

      const args = (input.args ?? {}) as Record<string, unknown>
      const text = argsText(args) + "\n" + outputText(output.output)

      if (PREFERRED_SEARCH_LIBRARY_RE.test(input.tool)) recordPreferredSearch(session, args, text)
      if (PREFERRED_ITEM_DETAILS_RE.test(input.tool)) recordPreferredItemDetails(session, args, text)
      if (RETRACTION_CHECK_RE.test(input.tool)) recordRetractionCheck(session, args, outputText(output.output))
    },

    "experimental.text.complete": async (_input, output) => {
      const session = state(_input.sessionID)
      if (!session.citationIntent) return
      if (!CITATION_MARKER_RE.test(output.text)) return

      const outputCitekeys = extractCitekeys(output.text)
      const unverified = outputCitekeys.filter((key) => !session.verifiedCitekeys.has(key))
      const citedKeys = outputCitekeys.length > 0 ? outputCitekeys : [...session.requestedCitekeys]
      const bibliographyOutput = BIBLIOGRAPHY_OUTPUT_RE.test(output.text)
      const missingNotes = missingLiteratureNotes(citedKeys, literatureNoteDir)
      const retractionUnchecked = bibliographyOutput
        ? citedKeys.filter((key) => session.verifiedCitekeys.has(key) && !session.retractionCheckedCitekeys.has(key))
        : []

      if (outputCitekeys.length > 0 && unverified.length === 0 && missingNotes.length === 0 && retractionUnchecked.length === 0) return

      const requestedButUnverified = [...session.requestedCitekeys].filter((key) => !session.verifiedCitekeys.has(key))
      const missing = unverified.length > 0 ? unverified : requestedButUnverified
      const warnings: string[] = []
      if (unverified.length > 0 || (outputCitekeys.length === 0 && missing.length > 0)) {
        warnings.push(`Unverified Zotero metadata citekeys: ${formatKeys(missing)}.`)
      }
      if (missingNotes.length > 0) {
        warnings.push(`Missing Obsidian literature notes in \`${literatureNoteDir}\`: ${formatKeys(missingNotes)}.`)
      }
      if (retractionUnchecked.length > 0) {
        warnings.push(`Bibliography/references output without recorded retraction/correction risk check: ${formatKeys(retractionUnchecked)}.`)
      }

      output.text = [
        `⚠️ Citation guard: ${warnings.length > 0 ? warnings.join(" ") : "this response contains citation/reference markers that need Zotero/Obsidian verification."} Treat affected citations as unverified before use.`,
        "",
        output.text,
      ].join("\n")
    },
  }
}) satisfies Plugin
