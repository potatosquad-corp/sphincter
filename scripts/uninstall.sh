#!/bin/bash
set -e

# Log Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== SPHINCTER UNINSTALLATION (Baltrou Server) ===${NC}"

# 1. Root privileges check
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Error: This script must be run as root (sudo).${NC}"
  exit 1
fi

echo -e "${GREEN}[1/3] Stopping and disabling service...${NC}"
if systemctl is-active --quiet sphincter.service; then
    systemctl stop sphincter.service
    echo "Service stopped."
else
    echo "Service was not running."
fi

if systemctl is-enabled --quiet sphincter.service; then
    systemctl disable sphincter.service
    echo "Service disabled."
fi

echo -e "${GREEN}[2/3] Removing system files...${NC}"
# Remove service file
if [ -f "/etc/systemd/system/sphincter.service" ]; then
    rm /etc/systemd/system/sphincter.service
    systemctl daemon-reload
    echo "Service file removed."
else
    echo "Service file not found."
fi

if [-f "/etc/default/sphincter"]; then
    rm /etc/default/sphincter
    echo "Config file removed"
else    
    echo "No config file found"
fi

echo -e "${GREEN}[3/3] Removing binary...${NC}"
# Remove binary
if [ -f "/usr/local/bin/sphincter" ]; then
    rm /usr/local/bin/sphincter
    echo "Binary removed from /usr/local/bin."
else
    echo "Binary not found."
fi

echo -e "${BLUE}=== UNINSTALLATION COMPLETE ===${NC}"
echo "Sphincter has been completely removed from your system."