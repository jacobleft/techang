# opencode-zotero-citation-guard

opencode plugin that adds a Zotero citation-integrity harness.

## What it does

- Injects citation-verification rules into the system prompt.
- Tracks citekeys mentioned in the conversation.
- Records verification when Zotero MCP metadata tools return complete metadata.
- Warns before final text when citekeys appear unverified.
- Optionally checks whether matching Obsidian literature notes exist.
- Warns when bibliography/reference output lacks a recorded retraction or correction check.

## Install

```json
{
  "plugin": [
    "opencode-zotero-citation-guard"
  ]
}
```

Restart opencode after changing `opencode.json`.

## Options

```json
{
  "plugin": [
    [
      "opencode-zotero-citation-guard",
      {
        "literatureNoteDir": "/Users/jacob/vaults/lifework/01_notes/0103_literature_notes"
      }
    ]
  ]
}
```

If `literatureNoteDir` is omitted, the plugin uses `ZOTERO_CITATION_GUARD_LITERATURE_NOTE_DIR` when set, otherwise it falls back to the path above.
