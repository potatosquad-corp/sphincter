#!/bin/bash
set -e

# Log Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== SPHINCTER INSTALLATION (Baltrou Server) ===${NC}"

# 1. Root privileges check
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Error: This script must be run as root (sudo).${NC}"
  exit 1
fi

# 2. Binary presence check
if [ ! -f "./sphincter" ]; then
    echo -e "${RED}Error: Binary file 'sphincter' not found in current directory.${NC}"
    echo "Please ensure you have extracted the full release archive."
    exit 1
fi

if [ ! -f /etc/default/sphincter ]; then
    echo "Creating default configuration file..."
    cat > /etc/default/sphincter <<EOF
# Sphincter config file
# Edit this file then restart the service : systemctl restart sphincter

# TCP listener address
SPHINCTER_TCP_ADDR=0.0.0.0:9000

# Websocket listener address
SPHINCTER_WS_ADDR=127.0.0.1:8080
EOF
fi

echo -e "${GREEN}[1/3] Installing binary...${NC}"
# Stop service if already running
systemctl stop sphincter.service 2>/dev/null || true

# Copy to /usr/local/bin
cp ./sphincter /usr/local/bin/sphincter
chmod +x /usr/local/bin/sphincter

echo -e "${GREEN}[2/3] Configuring Systemd service...${NC}"

# Create service file
cat > /etc/systemd/system/sphincter.service <<EOF
[Unit]
Description=Sphincter - TCP/WebSocket Relay
After=network.target
StartLimitBurst=3

[Service]
Type=simple
User=root
EnvironmentFile=/etc/default/sphincter
ExecStart=/usr/local/bin/sphincter
Restart=always
RestartSec=3
# Increase file descriptors limit to handle high concurrency
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

echo -e "${GREEN}[3/3] Starting service...${NC}"
systemctl daemon-reload
systemctl enable sphincter.service
systemctl restart sphincter.service

echo -e "${BLUE}=== INSTALLATION COMPLETE ===${NC}"
echo -e "Service Status:"
systemctl status sphincter.service --no-pager | head -n 10
echo -e "${BLUE}View real-time logs: ${NC}journalctl -u sphincter -f"