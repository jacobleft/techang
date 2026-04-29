# LaTeX Optimization Tools Installation Guide

Detailed platform-specific installation instructions for LaTeX tools that reduce token usage and improve error clarity for AI agents.

## Quick Install by Platform

### macOS (Homebrew)

```bash
# Option 1: Full installation (recommended)
brew install texlive pplatex

# Option 2: Minimal installation
brew install texlive

# Option 3: Modern alternative (easiest)
brew install tectonic
```

### Ubuntu/Debian

```bash
# Full installation with all tools
sudo apt-get update
sudo apt-get install -y texlive-full pplatex latexmk

# Minimal installation
sudo apt-get install -y texlive-latex-base texlive-latex-extra latexmk

# Just the modern engine
sudo apt-get install -y tectonic
```

### Fedora/RHEL/CentOS

```bash
# Full installation
sudo dnf install -y texlive-scheme-full pplatex latexmk

# Minimal installation
sudo dnf install -y texlive-latex texlive-collection-latexextra latexmk
```

### Arch Linux

```bash
# Full installation
sudo pacman -S texlive-most texlive-lang pplatex latexmk

# Minimal installation
sudo pacman -S texlive-bin texlive-latexextra latexmk
```

### Windows

#### With TeX Live
1. Download and install TeX Live from https://tug.org/texlive/
2. `texfot` and `latexmk` are included automatically
3. Download pplatex from: https://github.com/stefanhepp/pplatex/releases

#### With MiKTeX
1. Download and install MiKTeX from https://miktex.org/
2. Install `latexmk` via MiKTeX Console
3. Note: `texfot` may not be available with MiKTeX

#### With Tectonic (Recommended for Windows)
1. Download from: https://github.com/tectonic-typesetting/tectonic/releases
2. Add to PATH or use full path

## Tool-Specific Installation

### texfot

Comes with TeX Live. If missing:

```bash
# macOS
brew install texlive

# Ubuntu/Debian
sudo apt-get install texlive-extra-utils

# Fedora
sudo dnf install texlive-texfot
```

### pplatex

```bash
# macOS with Homebrew
brew install pplatex

# Ubuntu/Debian (24.04+)
sudo apt-get install pplatex

# Build from source (any platform)
git clone https://github.com/stefanhepp/pplatex.git
cd pplatex
mkdir build && cd build
cmake ..
make
sudo make install
```

### latexmk

```bash
# Usually included with TeX Live
# If missing:

# macOS
brew install latexmk

# Ubuntu/Debian
sudo apt-get install latexmk

# Fedora
sudo dnf install latexmk

# CPAN (any platform with Perl)
cpan install latexmk
```

### Tectonic

```bash
# macOS
brew install tectonic

# Linux with cargo
cargo install tectonic

# Linux without cargo
# Download binary from GitHub releases
wget https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%400.15.0/tectonic-0.15.0-x86_64-unknown-linux-gnu.tar.gz
tar xzf tectonic-*.tar.gz
sudo mv tectonic /usr/local/bin/

# Windows
# Download .exe from GitHub releases and add to PATH
```

## Docker Installation

For consistent environments across agents:

```dockerfile
FROM texlive/texlive:latest

# Install additional tools
RUN apt-get update && apt-get install -y \
    pplatex \
    latexmk \
    && rm -rf /var/lib/apt/lists/*

# Copy detection script
COPY scripts/detect-and-compile.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/detect-and-compile.sh

WORKDIR /workspace
```

Usage:
```bash
docker build -t latex-optimizer .
docker run -v $(pwd):/workspace latex-optimizer detect-and-compile.sh document.tex
```

## CI/CD Installation

### GitHub Actions

```yaml
- name: Install LaTeX tools
  run: |
    sudo apt-get update
    sudo apt-get install -y texlive-full pplatex latexmk
    
- name: Compile document
  run: |
    bash scripts/detect-and-compile.sh main.tex
```

### GitLab CI

```yaml
compile:
  image: texlive/texlive:latest
  before_script:
    - apt-get update && apt-get install -y pplatex latexmk
  script:
    - bash scripts/detect-and-compile.sh main.tex
  artifacts:
    paths:
      - main.pdf
```

## Verification

After installation, verify tools work:

```bash
# Check all tools
bash scripts/detect-and-compile.sh --detect

# Or check individually
texfot --version
latexmk --version
pplatex --version
pdflatex --version
tectonic --version
```

## Troubleshooting

### "command not found" after installation

```bash
# Reload shell configuration
source ~/.bashrc  # or ~/.zshrc

# Check PATH
echo $PATH

# Find binary location
which texfot || find /usr -name "texfot" 2>/dev/null
```

### Permission denied

```bash
# Fix permissions
chmod +x /path/to/script

# Or run with bash explicitly
bash /path/to/detect-and-compile.sh
```

### Outdated packages

```bash
# Update TeX Live
sudo tlmgr update --self --all

# Update Homebrew packages
brew update && brew upgrade texlive

# Update tectonic
cargo install --force tectonic
```

## Minimal Setup for AI Agents

If you want the absolute minimum installation for AI agent use:

```bash
# Option 1: Tectonic only (~50MB, self-contained)
brew install tectonic  # macOS
cargo install tectonic # Linux

# Option 2: Basic TeX Live + latexmk
sudo apt-get install texlive-latex-base latexmk  # Ubuntu

# Then use:
bash scripts/detect-and-compile.sh --tier=3 file.tex
```

## Size Comparison

| Installation | Size | Tools Included | Best For |
|-------------|------|----------------|----------|
| Tectonic only | ~50MB | tectonic | Quick setup, CI/CD |
| Minimal TeX Live | ~200MB | pdflatex, latexmk | Basic documents |
| Full TeX Live | ~4GB | All tools | Complex documents |
| texlive-full | ~6GB | Everything | Complete coverage |

## Recommended Setup by Use Case

### AI Agent Development (Local)
```bash
brew install texlive pplatex  # macOS
sudo apt-get install texlive-full pplatex  # Linux
```

### CI/CD Pipeline
```bash
# Use tectonic for speed
brew install tectonic  # macOS
cargo install tectonic # Linux
```

### Shared Development Environment
```bash
# Docker with full tools
docker pull texlive/texlive:latest
docker run -it texlive/texlive bash
```

### Minimal Quick Start
```bash
# Just need to compile? Use tectonic
brew install tectonic  # 1 minute install
tectonic file.tex       # Auto-downloads packages
```
