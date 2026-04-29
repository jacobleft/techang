# Example AI Agent Workflow Integration

This document shows how to integrate LaTeX optimization into AI agent workflows.

## Basic Workflow

```
User: "Compile this LaTeX document and fix any errors"

Agent:
1. Run detection to find available tools
2. Compile with optimal settings
3. Parse structured errors
4. Fix errors in source
5. Re-compile until successful
```

## Implementation

### Step 1: Detect Tools

```bash
# In your agent's working directory
RESULT=$(bash scripts/detect-and-compile.sh --detect --json)
TIER=$(echo "$RESULT" | grep -o '"detected_tier": [0-9]*' | grep -o '[0-9]*')
```

### Step 2: Compile with Error Capture

```bash
# Compile and capture output
OUTPUT=$(bash scripts/detect-and-compile.sh document.tex 2>&1)
EXIT_CODE=$?
```

### Step 3: Parse Errors (if compilation failed)

```bash
# If using pplatex (Tier 2), output is already structured
# If using texfot (Tier 1), errors are already filtered
# For other tiers, parse from log:

if [ $EXIT_CODE -ne 0 ]; then
    ERRORS=$(grep -A2 "^!" document.log)
    echo "Compilation errors found:"
    echo "$ERRORS"
fi
```

### Step 4: Iterative Fix Loop

```bash
MAX_RETRIES=3
RETRY=0

while [ $RETRY -lt $MAX_RETRIES ]; do
    bash scripts/detect-and-compile.sh document.tex
    
    if [ $? -eq 0 ]; then
        echo "Compilation successful!"
        break
    fi
    
    # Extract errors and ask agent to fix
    ERRORS=$(grep -A3 "^!" document.log | head -20)
    echo "Errors: $ERRORS"
    
    # Agent fixes errors here...
    
    RETRY=$((RETRY + 1))
done
```

## Integration with Different Agents

### Claude Code

Add to your project's CLAUDE.md:

```markdown
## LaTeX Compilation

When compiling LaTeX documents:
1. First run: `bash scripts/detect-and-compile.sh --detect`
2. Use the detected optimal command
3. If errors occur, they will be clearly visible in filtered output
4. Never use raw `pdflatex` without flags - always use the wrapper
```

### OpenCode

The skill is automatically available. When LaTeX compilation is needed, the skill triggers and provides the optimal compilation strategy.

### Cursor

Add to `.cursorrules`:

```
When working with LaTeX:
- Always use the latex-optimizer skill for compilation
- Run detection first: bash scripts/detect-and-compile.sh --detect
- Use structured error output to identify issues quickly
```

### GitHub Copilot Chat

Instruct Copilot:

```
Use the LaTeX optimization script at scripts/detect-and-compile.sh
when compiling LaTeX documents. It reduces output tokens by 90%+
and makes errors clearly visible.
```

## Makefile Integration

```makefile
LATEX_SCRIPT := scripts/detect-and-compile.sh

# Auto-detect on first run
.latex-detected:
	@echo "Detecting LaTeX tools..."
	@$(LATEX_SCRIPT) --detect > /dev/null
	@touch .latex-detected

%.pdf: %.tex .latex-detected
	$(LATEX_SCRIPT) $<

# Force detection refresh
detect:
	@rm -f .latex-detected
	@$(LATEX_SCRIPT) --detect

.PHONY: detect
```

## Python Integration

```python
import subprocess
import json

def compile_latex(file_path, force_tier=None):
    """Compile LaTeX with optimal settings."""
    
    # Detect available tools
    detect_cmd = ['bash', 'scripts/detect-and-compile.sh', '--detect', '--json']
    result = subprocess.run(detect_cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        # Fallback to basic compilation
        return subprocess.run(['pdflatex', file_path])
    
    detection = json.loads(result.stdout)
    tier = force_tier or detection['detected_tier']
    
    # Compile with detected tier
    compile_cmd = ['bash', 'scripts/detect-and-compile.sh', f'--tier={tier}', file_path]
    return subprocess.run(compile_cmd)

# Usage
result = compile_latex('document.tex')
if result.returncode != 0:
    print("Compilation failed - check output above for errors")
```

## Node.js Integration

```javascript
const { execSync } = require('child_process');

function compileLatex(filePath, forceTier = null) {
    // Detect tools
    const detection = JSON.parse(
        execSync('bash scripts/detect-and-compile.sh --detect --json', { encoding: 'utf8' })
    );
    
    const tier = forceTier || detection.detected_tier;
    
    // Compile
    try {
        execSync(`bash scripts/detect-and-compile.sh --tier=${tier} ${filePath}`, {
            stdio: 'inherit'
        });
        return { success: true };
    } catch (error) {
        return { success: false, error: error.message };
    }
}

// Usage
const result = compileLatex('document.tex');
if (!result.success) {
    console.error('Compilation failed');
}
```

## Docker Compose Setup

```yaml
version: '3'
services:
  latex:
    build: .
    volumes:
      - ./:/workspace
    command: bash scripts/detect-and-compile.sh document.tex
```

## CI/CD Pipeline Example

### GitHub Actions

```yaml
name: Compile LaTeX

on: [push]

jobs:
  compile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install LaTeX tools
        run: |
          sudo apt-get update
          sudo apt-get install -y texlive-full pplatex latexmk
      
      - name: Detect and compile
        run: |
          bash scripts/detect-and-compile.sh --detect
          bash scripts/detect-and-compile.sh main.tex
      
      - name: Upload PDF
        uses: actions/upload-artifact@v3
        with:
          name: document
          path: main.pdf
```

## Best Practices

1. **Always detect first** - Don't assume tools are available
2. **Use structured output** - Parse JSON detection for programmatic use
3. **Handle failures gracefully** - Fall back to lower tiers if needed
4. **Cache detection results** - Don't re-detect on every compilation
5. **Commit the script** - Include `scripts/detect-and-compile.sh` in your repository

## Example Session

```
User: "I need to compile my thesis. Can you help?"

Agent: "I'll help you compile your thesis with optimized settings to reduce 
token usage and make errors clear."

Agent runs: bash scripts/detect-and-compile.sh --detect

Output:
=== LaTeX Tool Detection Results ===
✓ texfot found (Tier 1)
✓ latexmk found (Tier 1)
✓ pplatex found (Tier 2)
✓ pdflatex found (Tier 4)

Recommended tier: Tier 1 - texfot + latexmk
Token reduction: ~95%
Error visibility: Excellent

Recommended command:
  texfot --tee=/dev/null --quiet latexmk -pdf -silent -interaction=nonstopmode -halt-on-error file.tex

Agent: "Great! I found texfot and latexmk. I'll compile with Tier 1 settings 
for maximum token reduction and clear error reporting."

Agent runs: bash scripts/detect-and-compile.sh thesis.tex

Output:
Compiling with Tier 1: texfot + latexmk

Latexmk: Run number 1 of rule 'pdflatex'
Latexmk: Running 'pdflatex -interaction=batchmode "thesis.tex"'
Latexmk: Run number 2 of rule 'pdflatex'
Latexmk: Running 'pdflatex -interaction=batchmode "thesis.tex"'
Latexmk: Run number 3 of rule 'pdflatex'
Latexmk: Running 'pdflatex -interaction=batchmode "thesis.tex"'
Latexmk: All runs completed

Agent: "Compilation successful! Your thesis.pdf has been generated with 
optimal settings. No errors were found."
```
