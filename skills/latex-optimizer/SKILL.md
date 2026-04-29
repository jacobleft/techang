---
name: latex-optimizer
description: Optimize LaTeX compilation for AI agents by reducing token usage and making errors/warnings clear. Auto-detects available tools (texfot, pplatex, latexmk, tectonic) and configures the best compilation strategy. Use whenever compiling LaTeX documents to minimize output tokens and maximize error clarity. Essential for AI-assisted LaTeX workflows in Claude Code, OpenCode, Cursor, or any agent environment.
---

# LaTeX Optimizer for AI Agents

A skill that makes LaTeX compilation AI-friendly by dramatically reducing output verbosity and surfacing errors clearly.

## Why This Matters

Default LaTeX compilation produces **500-2000 lines of output** per run, with errors buried in noise. For AI agents, this wastes tokens and makes debugging difficult. This skill auto-detects and configures the best available tools to reduce output by **90-99%** while making errors immediately visible.

## Quick Start

Run the detection script to see what's available and get optimized commands:

```bash
# Auto-detect and show recommended compilation command
bash scripts/detect-and-compile.sh --detect

# Compile a document with optimal settings
bash scripts/detect-and-compile.sh document.tex
```

## How It Works

The skill uses a **tiered fallback system**:

1. **Tier 1** (Best): `texfot` + `latexmk` - Filters output to show only errors/warnings
2. **Tier 2** (Good): `pplatex` - Parses log files into structured error format  
3. **Tier 3** (Basic): `latexmk -silent` - Suppresses most output, handles multiple passes
4. **Tier 4** (Minimal): `pdflatex -interaction=batchmode` - Maximum suppression
5. **Tier 5** (Fallback): Plain `pdflatex` with grep filtering

## Auto-Detection Script

The script `scripts/detect-and-compile.sh` handles everything:

```bash
# Detect available tools and show configuration
bash scripts/detect-and-compile.sh --detect

# Compile with best available method
bash scripts/detect-and-compile.sh file.tex

# Force specific tier
bash scripts/detect-and-compile.sh --tier=1 file.tex
bash scripts/detect-and-compile.sh --tier=4 file.tex

# Show help
bash scripts/detect-and-compile.sh --help
```

### What the Script Does

1. Checks for each tool in PATH
2. Tests if tools actually work (not just exist)
3. Selects the best available tier
4. Configures compilation command accordingly
5. If no tools found, prompts user with options

### Detection Output Example

```
=== LaTeX Tool Detection Results ===
✓ texfot found (Tier 1)
✓ latexmk found (Tier 1)
✗ pplatex not found (Tier 2)
✓ tectonic found (Tier 3 alternative)

Recommended command:
  texfot --tee=/dev/null --quiet latexmk -pdf -silent -interaction=nonstopmode file.tex

Token reduction: ~95%
Error visibility: Excellent
```

## When No Tools Are Found

If no optimization tools are detected, the script will:

1. **Show current status** with clear ❌ marks for missing tools
2. **Offer fallback options**:
   - Use plain `pdflatex` with manual grep filtering
   - Use `latexmk` basic mode (if available)
   - Continue with raw output (not recommended)
3. **Provide installation guidance** for your platform

Example prompt when nothing is found:

```
⚠️  No LaTeX optimization tools detected!

Available options:
1. Use plain pdflatex with basic filtering (no installation needed)
2. Install recommended tools for 90%+ token reduction
3. Continue with raw LaTeX output (high token usage)

Choose an option (1-3): 
```

## Tool Installation Guide

### macOS (Homebrew)

```bash
# Install all recommended tools at once
brew install texlive  # Includes latexmk, texfot
brew install pplatex  # Optional but recommended

# Or install minimal set
brew install tectonic  # Modern alternative, self-contained
```

### Linux (Debian/Ubuntu)

```bash
# Full TeX Live with all tools
sudo apt-get install texlive-full pplatex latexmk

# Or minimal
sudo apt-get install texlive-latex-base texlive-latex-extra latexmk
```

### Linux (Fedora/RHEL)

```bash
sudo dnf install texlive-scheme-full pplatex latexmk
```

### Windows

```powershell
# Install TeX Live or MiKTeX first, then:
# - texfot and latexmk come with TeX Live
# - pplatex: download from https://github.com/stefanhepp/pplatex
```

### Using Tectonic (Cross-platform, Easiest)

```bash
# macOS
brew install tectonic

# Linux
cargo install tectonic

# No package manager needed - self-contained
```

## Manual Configuration

If you prefer to configure compilation manually, here are the optimal commands per tier:

### Tier 1: texfot + latexmk (Recommended)

```bash
# Show only errors and warnings
# Reduces output from ~1000 lines to ~10-30 lines
texfot --tee=/dev/null --quiet latexmk -pdf -silent -interaction=nonstopmode -halt-on-error file.tex

# With specific engine
texfot --tee=/dev/null latexmk -pdf -pdflatex="pdflatex -file-line-error" -silent file.tex
```

### Tier 2: pplatex (Structured Output)

```bash
# Compile then parse log
pdflatex -interaction=nonstopmode file.tex
pplatex -i file.log

# Or pipe directly
pdflatex file.tex | pplatex -i

# Using ppdflatex wrapper (if available)
ppdflatex file.tex  # Compiles + parses in one step
```

Output format:
```
** Warning in ./file.tex: No file file.aux.
** Error in ./file.tex, Line 9: Undefined control sequence \unknown

Result: o) Errors: 1, Warnings: 1, BadBoxes: 0
```

### Tier 3: latexmk -silent

```bash
# Suppresses most output, auto-handles multiple passes
latexmk -pdf -silent -interaction=nonstopmode -halt-on-error file.tex

# Even quieter
latexmk -pdf -silent -interaction=batchmode file.tex
```

### Tier 4: Direct Engine Batchmode

```bash
# Maximum suppression - only fatal errors
pdflatex -interaction=batchmode -halt-on-error file.tex

# Slightly more verbose but shows errors
pdflatex -interaction=nonstopmode -halt-on-error file.tex

# Combine with output redirection
pdflatex -interaction=nonstopmode file.tex 2>&1 | grep -E "^(!|l\.|Error|Warning)"
```

### Tier 5: Plain with Filtering

```bash
# If nothing else is available, use grep to filter raw output
pdflatex file.tex 2>&1 | grep -E "^(!|l\.[0-9]+|.*Error|.*Warning|.*Overfull|.*Underfull)"

# Or save and parse log
pdflatex file.tex && grep -E "^(!|l\.)" file.log
```

## Integration Examples

### In AI Agent Workflows

When an AI agent needs to compile LaTeX, use this pattern:

```bash
# 1. Detect available tools
DETECTED=$(bash scripts/detect-and-compile.sh --detect --json)

# 2. Compile with detected optimal settings
bash scripts/detect-and-compile.sh file.tex

# 3. If compilation fails, errors are already filtered and clear
```

### In Makefile

```makefile
# Auto-detect and use best available compiler
LATEX_COMPILER := $(shell bash scripts/detect-and-compile.sh --detect --command)

%.pdf: %.tex
	$(LATEX_COMPILER) $<

# Or explicit tier
pdf:
	bash scripts/detect-and-compile.sh --tier=1 main.tex
```

### In CI/CD (GitHub Actions)

```yaml
- name: Compile LaTeX
  run: |
    # Install tools if not present
    if ! command -v texfot &> /dev/null; then
      sudo apt-get install -y texlive-extra-utils pplatex
    fi
    
    # Compile with optimization
    bash scripts/detect-and-compile.sh main.tex
```

## Understanding the Output

### With texfot (Tier 1)

Shows only actionable messages:
- Error messages (`! Undefined control sequence`)
- Overfull/underfull boxes
- Undefined citations/references
- Missing font characters

### With pplatex (Tier 2)

Structured format:
```
** Error in file.tex, Line 42: Missing $ inserted
** Warning in file.tex: Reference `fig:1' undefined
Result: o) Errors: 1, Warnings: 1, BadBoxes: 0
```

### With latexmk -silent (Tier 3)

Shows compilation progress but suppresses LaTeX noise:
```
Latexmk: Run number 1 of rule 'pdflatex'
Latexmk: Running 'pdflatex -interaction=batchmode "file.tex"'
Latexmk: Run number 2 of rule 'pdflatex'
Latexmk: Running 'pdflatex -interaction=batchmode "file.tex"'
Latexmk: Run number 3 of rule 'pdflatex'
Latexmk: Running 'pdflatex -interaction=batchmode "file.tex"'
Latexmk: All runs completed
```

## Troubleshooting

### "texfot not found" but TeX Live is installed

```bash
# texfot is in texlive-extra-utils on some distributions
sudo apt-get install texlive-extra-utils  # Debian/Ubuntu
```

### "pplatex not found" 

```bash
# Build from source if package not available
git clone https://github.com/stefanhepp/pplatex.git
cd pplatex && mkdir build && cd build && cmake .. && make && sudo make install
```

### Tectonic has different output format

Tectonic uses non-traditional output. Use this wrapper:
```bash
tectonic file.tex 2>&1 | grep -iE "(error|warning|^!)"
```

### Compilation hangs (waits for input)

Always use `-interaction=nonstopmode` or `-interaction=batchmode` to prevent interactive prompts.

## Performance Comparison

| Method | Typical Output Lines | Token Reduction | Error Clarity |
|--------|---------------------|-----------------|---------------|
| Default pdflatex | 500-2000 | Baseline | Poor |
| latexmk -silent | 50-100 | ~80% | Moderate |
| texfot filtering | 10-30 | ~95% | Good |
| pplatex parsing | 5-15 | ~98% | Excellent |
| batchmode + log | 0-5 | ~99% | Excellent |

## References

- `references/install-guide.md` - Detailed platform-specific installation instructions
- `assets/example-workflow.md` - Example AI agent workflow integration

## Compatibility

Works with:
- Claude Code / OpenCode
- Cursor
- GitHub Copilot Chat
- Any AI agent that can execute shell commands

Requirements:
- Bash shell
- Standard Unix tools (grep, sed, awk)
- At least one LaTeX distribution (TeX Live, MiKTeX, or Tectonic)
