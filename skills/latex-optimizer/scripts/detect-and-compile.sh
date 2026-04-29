#!/bin/bash
#
# detect-and-compile.sh - Auto-detect LaTeX optimization tools and compile documents
#
# This script detects available LaTeX tools and configures the optimal
# compilation strategy to minimize output tokens and maximize error clarity.
#
# Usage:
#   bash detect-and-compile.sh [options] [file.tex]
#
# Options:
#   --detect          Show detection results and recommended command
#   --detect --json   Output detection results as JSON
#   --tier=N          Force specific tier (1-5)
#   --help            Show this help message
#
# Tiers:
#   1. texfot + latexmk    (Best: ~95% token reduction, excellent errors)
#   2. pplatex             (Good: structured error output)
#   3. latexmk -silent     (Basic: suppresses most output)
#   4. pdflatex batchmode  (Minimal: maximum suppression)
#   5. pdflatex + grep     (Fallback: basic filtering)

set -euo pipefail

# Colors for output (disable if not terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' BOLD='' RESET=''
fi

# Tier definitions
TIER_NAMES=(
    "texfot + latexmk"
    "pplatex"
    "latexmk -silent"
    "pdflatex batchmode"
    "pdflatex + grep"
)

TIER_REDUCTION=(
    "~95%"
    "~98%"
    "~80%"
    "~99%"
    "~70%"
)

TIER_CLARITY=(
    "Excellent"
    "Excellent"
    "Moderate"
    "Good"
    "Basic"
)

# Global variables
DETECTED_TIER=0
DETECTED_CMD=""
TEX_FILE=""
MODE="compile"
FORCE_TIER=0
OUTPUT_JSON=false

# Function to check if a command exists and works
command_works() {
    local cmd="$1"
    if ! command -v "$cmd" &> /dev/null; then
        return 1
    fi
    
    # Test if the command actually runs (some might be broken scripts)
    case "$cmd" in
        texfot)
            texfot --version &> /dev/null || return 1
            ;;
        latexmk)
            latexmk --version &> /dev/null || return 1
            ;;
        pplatex)
            pplatex --version &> /dev/null || pplatex -h &> /dev/null || return 1
            ;;
        tectonic)
            tectonic --version &> /dev/null || return 1
            ;;
        pdflatex|xelatex|lualatex)
            "$cmd" --version &> /dev/null || return 1
            ;;
    esac
    
    return 0
}

# Function to detect available tools
detect_tools() {
    local tier1=false
    local tier2=false
    local tier3=false
    local tier4=false
    local tier5=false
    local has_pdflatex=false
    local tectonic_alt=false
    
    echo "=== LaTeX Tool Detection Results ==="
    echo ""
    
    # Tier 1: texfot + latexmk
    if command_works "texfot" && command_works "latexmk"; then
        tier1=true
        echo -e "${GREEN}✓${RESET} texfot found (Tier 1)"
        echo -e "${GREEN}✓${RESET} latexmk found (Tier 1)"
    else
        if ! command_works "texfot"; then
            echo -e "${RED}✗${RESET} texfot not found (Tier 1)"
        fi
        if ! command_works "latexmk"; then
            echo -e "${RED}✗${RESET} latexmk not found (Tier 1)"
        fi
    fi
    
    # Tier 2: pplatex
    if command_works "pplatex" || command_works "ppdflatex"; then
        tier2=true
        echo -e "${GREEN}✓${RESET} pplatex found (Tier 2)"
    else
        echo -e "${RED}✗${RESET} pplatex not found (Tier 2)"
    fi
    
    # Tier 3: latexmk -silent
    if command_works "latexmk"; then
        tier3=true
        echo -e "${GREEN}✓${RESET} latexmk found (Tier 3)"
    fi
    
    # Tier 4: pdflatex with batchmode
    if command_works "pdflatex"; then
        tier4=true
        has_pdflatex=true
        echo -e "${GREEN}✓${RESET} pdflatex found (Tier 4)"
    else
        echo -e "${RED}✗${RESET} pdflatex not found (Tier 4)"
    fi
    
    # Check for alternative engines
    if command_works "tectonic"; then
        tectonic_alt=true
        echo -e "${GREEN}✓${RESET} tectonic found (alternative)"
    fi
    
    # Tier 5: always available if pdflatex exists
    if [ "$has_pdflatex" = true ]; then
        tier5=true
    fi
    
    echo ""
    
    # Determine best tier
    if [ "$tier1" = true ]; then
        DETECTED_TIER=1
    elif [ "$tier2" = true ]; then
        DETECTED_TIER=2
    elif [ "$tier3" = true ]; then
        DETECTED_TIER=3
    elif [ "$tier4" = true ]; then
        DETECTED_TIER=4
    elif [ "$tier5" = true ]; then
        DETECTED_TIER=5
    else
        DETECTED_TIER=0
    fi
    
    # Build command based on tier
    build_compile_command "$DETECTED_TIER"
    
    # Show results
    if [ "$DETECTED_TIER" -gt 0 ]; then
        echo -e "${GREEN}${BOLD}Recommended tier:${RESET} ${BOLD}Tier $DETECTED_TIER${RESET} - ${TIER_NAMES[$((DETECTED_TIER-1))]}"
        echo ""
        echo "Token reduction: ${TIER_REDUCTION[$((DETECTED_TIER-1))]}"
        echo "Error visibility: ${TIER_CLARITY[$((DETECTED_TIER-1))]}"
        echo ""
        echo -e "${BLUE}Recommended command:${RESET}"
        echo "  $DETECTED_CMD file.tex"
        
        if [ "$tectonic_alt" = true ] && [ "$DETECTED_TIER" -ge 4 ]; then
            echo ""
            echo -e "${YELLOW}Alternative:${RESET} tectonic file.tex (self-contained, no package management)"
        fi
    else
        show_no_tools_message
    fi
}

# Function to build compilation command for a tier
build_compile_command() {
    local tier="$1"
    local file="${2:-file.tex}"
    
    case "$tier" in
        1)
            DETECTED_CMD="texfot --tee=/dev/null --quiet latexmk -pdf -silent -interaction=nonstopmode -halt-on-error"
            ;;
        2)
            DETECTED_CMD="ppdflatex -q --"
            ;;
        3)
            DETECTED_CMD="latexmk -pdf -silent -interaction=nonstopmode -halt-on-error"
            ;;
        4)
            DETECTED_CMD="pdflatex -interaction=batchmode -halt-on-error"
            ;;
        5)
            DETECTED_CMD="pdflatex -interaction=nonstopmode -halt-on-error"
            ;;
        *)
            DETECTED_CMD=""
            ;;
    esac
}

# Function to show message when no tools found
show_no_tools_message() {
    echo -e "${YELLOW}${BOLD}⚠️  No LaTeX optimization tools detected!${RESET}"
    echo ""
    echo "Your system appears to be missing LaTeX tools."
    echo ""
    echo -e "${BOLD}Options:${RESET}"
    echo ""
    echo -e "${BOLD}1.${RESET} Use plain pdflatex with basic filtering"
    echo "   Command: pdflatex -interaction=nonstopmode file.tex | grep -E '^(!|l\.)'"
    echo ""
    echo -e "${BOLD}2.${RESET} Install recommended tools for 90%+ token reduction"
    echo "   See: references/install-guide.md"
    echo ""
    echo -e "${BOLD}3.${RESET} Install tectonic (easiest option)"
    echo "   macOS: brew install tectonic"
    echo "   Linux: cargo install tectonic"
    echo ""
    echo -e "${BOLD}4.${RESET} Continue with raw LaTeX output (not recommended)"
    echo "   Command: pdflatex file.tex"
    echo ""
}

# Function to prompt user for action when no tools found
prompt_user_choice() {
    echo ""
    echo -e "${YELLOW}What would you like to do?${RESET}"
    echo ""
    echo "1. Install tectonic (modern, self-contained LaTeX engine)"
    echo "2. Install texlive-full (complete LaTeX distribution with all tools)"
    echo "3. Use plain pdflatex with basic filtering"
    echo "4. Show detailed installation guide"
    echo "5. Exit"
    echo ""
    
    # In non-interactive mode (piped input), default to option 3
    if [ ! -t 0 ]; then
        echo "Non-interactive mode detected. Using option 3 (plain pdflatex with filtering)."
        return 3
    fi
    
    read -p "Choose an option (1-5): " choice
    echo ""
    
    case "$choice" in
        1)
            install_tectonic
            return 1
            ;;
        2)
            install_texlive
            return 2
            ;;
        3)
            return 3
            ;;
        4)
            show_install_guide
            return 4
            ;;
        5)
            exit 0
            ;;
        *)
            echo "Invalid choice. Using default (plain pdflatex with filtering)."
            return 3
            ;;
    esac
}

# Function to install tectonic
install_tectonic() {
    echo -e "${BLUE}Installing tectonic...${RESET}"
    
    if command -v brew &> /dev/null; then
        brew install tectonic
    elif command -v cargo &> /dev/null; then
        cargo install tectonic
    elif command -v apt-get &> /dev/null; then
        echo "Please install cargo first: sudo apt-get install cargo"
        echo "Then run: cargo install tectonic"
    elif command -v dnf &> /dev/null; then
        echo "Please install cargo first: sudo dnf install cargo"
        echo "Then run: cargo install tectonic"
    else
        echo "Could not detect package manager. Please visit: https://tectonic-typesetting.github.io/"
    fi
}

# Function to install texlive
install_texlive() {
    echo -e "${BLUE}Installing TeX Live...${RESET}"
    
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y texlive-full latexmk pplatex
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y texlive-scheme-full latexmk pplatex
    elif command -v brew &> /dev/null; then
        brew install texlive pplatex
    else
        echo "Could not detect package manager."
        echo "Please visit: https://tug.org/texlive/"
    fi
}

# Function to show installation guide
show_install_guide() {
    echo ""
    echo -e "${BOLD}=== Installation Guide ===${RESET}"
    echo ""
    echo "For detailed installation instructions, see:"
    echo "  references/install-guide.md"
    echo ""
    echo "Quick platform-specific commands:"
    echo ""
    echo "macOS (Homebrew):"
    echo "  brew install texlive pplatex"
    echo "  or: brew install tectonic"
    echo ""
    echo "Ubuntu/Debian:"
    echo "  sudo apt-get install texlive-full pplatex latexmk"
    echo ""
    echo "Fedora/RHEL:"
    echo "  sudo dnf install texlive-scheme-full pplatex latexmk"
    echo ""
    echo "Windows (with TeX Live installed):"
    echo "  Download pplatex from: https://github.com/stefanhepp/pplatex"
    echo ""
}

# Function to compile document
compile_document() {
    local tier="$1"
    local file="$2"
    
    if [ ! -f "$file" ]; then
        echo -e "${RED}Error: File not found: $file${RESET}"
        exit 1
    fi
    
    echo -e "${BLUE}Compiling with Tier $tier: ${TIER_NAMES[$((tier-1))]}${RESET}"
    echo ""
    
    case "$tier" in
        1)
            # texfot + latexmk
            texfot --tee=/dev/null --quiet latexmk -pdf -silent -interaction=nonstopmode -halt-on-error "$file"
            ;;
        2)
            # pplatex
            if command -v ppdflatex &> /dev/null; then
                ppdflatex -q -- "$file"
            else
                pdflatex -interaction=nonstopmode "$file" | pplatex -i
            fi
            ;;
        3)
            # latexmk -silent
            latexmk -pdf -silent -interaction=nonstopmode -halt-on-error "$file"
            ;;
        4)
            # pdflatex batchmode
            pdflatex -interaction=batchmode -halt-on-error "$file"
            ;;
        5)
            # pdflatex + grep filtering
            pdflatex -interaction=nonstopmode -halt-on-error "$file" 2>&1 | \
                grep -E "^(!|l\.[0-9]+|.*Error|.*Warning|.*Overfull|.*Underfull|.*BadBox)" || true
            ;;
        *)
            # Fallback - raw output (shouldn't reach here)
            pdflatex "$file"
            ;;
    esac
}

# Function to output JSON detection results
output_json() {
    local tier="$DETECTED_TIER"
    
    echo "{"
    echo "  \"detected_tier\": $tier,"
    echo "  \"tier_name\": \"${TIER_NAMES[$((tier-1))]:-none}\","
    echo "  \"token_reduction\": \"${TIER_REDUCTION[$((tier-1))]:-none}\","
    echo "  \"error_clarity\": \"${TIER_CLARITY[$((tier-1))]:-none}\","
    echo "  \"compile_command\": \"$DETECTED_CMD\","
    echo "  \"tools\": {"
    echo "    \"texfot\": $(command_works "texfot" && echo "true" || echo "false"),"
    echo "    \"latexmk\": $(command_works "latexmk" && echo "true" || echo "false"),"
    echo "    \"pplatex\": $(command_works "pplatex" && echo "true" || echo "false"),"
    echo "    \"pdflatex\": $(command_works "pdflatex" && echo "true" || echo "false"),"
    echo "    \"tectonic\": $(command_works "tectonic" && echo "true" || echo "false")"
    echo "  }"
    echo "}"
}

# Function to show help
show_help() {
    cat << 'EOF'
LaTeX Optimizer for AI Agents

Usage:
  bash detect-and-compile.sh [options] [file.tex]

Options:
  --detect              Show detection results and recommended command
  --detect --json       Output detection results as JSON
  --tier=N              Force specific tier (1-5)
  --help                Show this help message

Tiers:
  1. texfot + latexmk    (Best: ~95% token reduction, excellent errors)
  2. pplatex             (Good: structured error output)
  3. latexmk -silent     (Basic: suppresses most output)
  4. pdflatex batchmode  (Minimal: maximum suppression)
  5. pdflatex + grep     (Fallback: basic filtering)

Examples:
  bash detect-and-compile.sh --detect
  bash detect-and-compile.sh --detect --json
  bash detect-and-compile.sh document.tex
  bash detect-and-compile.sh --tier=1 document.tex
  bash detect-and-compile.sh --tier=4 document.tex

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --detect)
            MODE="detect"
            shift
            ;;
        --json)
            OUTPUT_JSON=true
            shift
            ;;
        --tier=*)
            FORCE_TIER="${1#*=}"
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        -*)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            TEX_FILE="$1"
            shift
            ;;
    esac
done

# Main logic
if [ "$MODE" = "detect" ]; then
    detect_tools
    
    if [ "$OUTPUT_JSON" = true ]; then
        output_json
    fi
    
    # If no tools found and not in JSON mode, prompt user
    if [ "$DETECTED_TIER" -eq 0 ] && [ "$OUTPUT_JSON" = false ]; then
        prompt_user_choice
        choice=$?
        
        case "$choice" in
            1|2)
                # User chose to install - re-detect
                echo ""
                echo "Re-detecting tools after installation..."
                detect_tools
                ;;
            3)
                # User chose plain pdflatex
                DETECTED_TIER=5
                build_compile_command 5
                echo -e "${YELLOW}Using fallback: Tier 5${RESET}"
                ;;
            4)
                # User wants to see guide, then exit
                exit 0
                ;;
        esac
    fi
    
    exit 0
fi

# Compilation mode
if [ -z "$TEX_FILE" ]; then
    echo "Error: No .tex file specified"
    show_help
    exit 1
fi

# Determine tier to use
if [ "$FORCE_TIER" -gt 0 ]; then
    USE_TIER="$FORCE_TIER"
    echo -e "${BLUE}Using forced tier: $USE_TIER${RESET}"
else
    # Auto-detect
    detect_tools > /dev/null 2>&1
    USE_TIER="$DETECTED_TIER"
fi

if [ "$USE_TIER" -eq 0 ]; then
    echo -e "${YELLOW}No optimization tools found. Falling back to basic mode.${RESET}"
    echo ""
    prompt_user_choice
    choice=$?
    
    case "$choice" in
        1|2)
            # Re-detect after installation
            detect_tools > /dev/null 2>&1
            USE_TIER="$DETECTED_TIER"
            ;;
        3)
            USE_TIER=5
            ;;
        *)
            exit 0
            ;;
    esac
fi

# Compile the document
compile_document "$USE_TIER" "$TEX_FILE"
