# Sphincter
*Socket Protocol Handler Interfacing Native Channels To Encapsulated Routes*

Sphincter is a high-performance, asynchronous relay server written in Rust, designed to bridge the gap between raw local TCP streams and the public web.


## 🔄 The Relay Architecture

Unlike a traditional reverse proxy that forwards traffic to an upstream server, Sphincter is the destination for both ends of the tunnel. It manages the state and distribution of data in memory.

1. **Ingest (TCP)**: Sphincter listens on a raw TCP port (Default: 9000).
2. **Dilate (Session Management)**: For every new TCP connection, Sphincter generates a unique, ephemeral 6-character Room ID (e.g., X7K9P2). This isolates the stream.
3. **Expel (WebSocket Broadcast)**: Sphincter simultaneously hosts a WebSocket server (Default: 8080). Web clients (browsers) connect to a specific room (/ws?room=CODE). Sphincter then broadcasts incoming TCP data to all subscribers of that room in real-time.

# ⚙️ Use Cases

1. OBS Remote Data: Sending local OBS WebSocket data to a remote web overlay hosted on a public server.
2. Secure Tunneling: Exposing a local TCP application to the web via WSS (Secure WebSocket) without exposing the application's native port directly to the internet.
3. One-to-Many Broadcasting: A single local source can be consumed by hundreds of web clients simultaneously through Sphincter's broadcast channel.

# Installation
1. Download the lastest `sphincter-linux-x64.tar.gz` archive from the Releases page
2. Run the provided installation script
```bash
# Run the installation script as root
sudo ./install.sh
```

This script will:
1. Copy the binary to `/usr/local/bin/sphincter`
2. Create the default config file (located at `/etc/default/sphincter`)
3. create the service `sphincter.service`
4. start/restart the service

Then it will be available on port **TCP 9000 (input)** and **HTTP 8080 (output)**.

## Uninstallation
To completely remove the server from your system, use the uninstallation script provided in the release
```bash
# Run the installation script as root
sudo ./uninstall.sh
```

## Nginx config
If you want to expose the output websocket as a secure websocket (WSS), add this block to your Nginx config
```
server {
    server_name your-domain.com;

    # ... standard SSL configuration (Certbot) ...

    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header X-Real-IP $remote_addr;
        
        # Timeouts to prevent disconnections
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}
```

## Change ports
You can find the config at `/etc/default/sphincter`, simply edit it then restart the service with `sudo systemctl restart sphincter.service`