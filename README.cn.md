# techang （特长）

**AI 代理技能包与 opencode 插件**，适用于 Claude Code、OpenCode、Cursor 等 AI 编程工具。

[English](README.md)

---

## 安装

```bash
# 全部安装
npx skills add jacobleft/techang

# 预览
npx skills add jacobleft/techang --list

# 只装一个
npx skills add jacobleft/techang --skill latex-optimizer
```

---

## 技能列表

| 技能 | 功能 |
|:-----|:-----|
| **intimate-relationship-guide** | 基于科学心理学的亲密关系指导——吸引力、沟通、冲突、维护、分手恢复 |
| **latex-optimizer** | AI 代理低噪音 LaTeX 编译；自动检测 `texfot`、`pplatex`、`latexmk`、`tectonic` |
| **latex-sentence-per-line** | 将 LaTeX 正文规范化为每行一句 |
| **ocr-tiered** | 分层 OCR：PaddleOCR（最高精度，需 Python 依赖）→ Tesseract + `tessdata_best`（平衡）→ 默认 Tesseract（快速、轻量） |
| **wittgenstein-authoring** | 审核并修订面向读者的 prose，处理防御性表述、术语漂移、结构薄弱、逻辑跳跃与过度声称 |

```
techang/
├── README.md
├── plugins/
│   ├── opencode-latex-sentence-per-line-harness/
│   └── opencode-zotero-citation-guard/
└── skills/
    ├── intimate-relationship-guide/
    ├── latex-optimizer/
    ├── latex-sentence-per-line/
    ├── ocr-tiered/
    └── wittgenstein-authoring/
```

---

## OpenCode 插件

| 包名 | 功能 |
|:-----|:-----|
| **opencode-zotero-citation-guard** | 为 Zotero 引文、citekey、文献笔记检查、参考文献撤稿检查提供护栏 |
| **opencode-latex-sentence-per-line-harness** | 在 edit/write/patch 工具修改 `.tex` 文件后，自动改写为每行一句 |

发布后安装：

```json
{
  "plugin": [
    "opencode-zotero-citation-guard",
    "opencode-latex-sentence-per-line-harness"
  ]
}
```

修改 `opencode.json` 后需要重启 opencode。
