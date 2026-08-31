#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
COMMAND_SOURCE="$SCRIPT_DIR/mambosite.sh"
INSTALL_DIR="${MAMBOSITE_BIN_DIR:-/usr/local/bin}"
COMMAND_TARGET="$INSTALL_DIR/mambosite"

chmod +x "$COMMAND_SOURCE"
if [[ -w "$INSTALL_DIR" ]]; then
    ln -sfn "$COMMAND_SOURCE" "$COMMAND_TARGET"
else
    sudo mkdir -p "$INSTALL_DIR"
    sudo ln -sfn "$COMMAND_SOURCE" "$COMMAND_TARGET"
fi

echo -e "${BLUE}------------------------------------------${NC}"
echo -e " Tool:    ${GREEN}MamboSite${NC}"
echo -e " Source:  $PROJECT_DIR"
echo -e " Command: $COMMAND_TARGET"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo -e "${YELLOW}[!] Add $INSTALL_DIR to PATH before using mambosite globally.${NC}" ;;
esac

echo -e "${GREEN}[+] Installation successful!${NC}"
echo -e "${BLUE}------------------------------------------${NC}"
