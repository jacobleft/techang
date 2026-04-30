# techang （特长）

**AI 代理技能包**，适用于 Claude Code、OpenCode、Cursor 等 Vercel `skills` CLI 支持的工具。

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

```
techang/
├── README.md
└── skills/
    ├── intimate-relationship-guide/
    ├── latex-optimizer/
    ├── latex-sentence-per-line/
    └── ocr-tiered/
```
