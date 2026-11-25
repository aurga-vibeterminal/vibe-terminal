`Host with HTTP server` publishes any directory on the server as an HTTP file service with one click: other devices on the same network (or over the relay) — a phone, another computer, a test rig — can browse and download that directory from a browser. Ideal for sharing build artifacts and log bundles, or handing files to devices without the client installed.

## How to use it

Three entries, all on a directory: the file list's right-click menu, a Recent entry's right-click menu, and the breadcrumbs' right-click menu → `Host with HTTP server`.

Clicking it:

1. The server starts a static HTTP service rooted at that directory, with the port taken from the Static HTTP port range;
2. **Local server**: the `http://127.0.0.1:port/` URL is copied straight to the clipboard, with a status message "HTTP server: … (copied to clipboard)";
3. **Remote server** (connected through a relay): the client automatically requests a port-forward mapping and copies the externally reachable share URL — LAN restrictions are not a blocker.

Open that URL in any device's browser and the directory browses and downloads like a normal web listing.

## Managing running services

Each HTTP service keeps running until stopped:

- Open the `Static HTTP` section of the Web UI; the `Services` list shows every running service and its document root;
- Each entry has a stop button; the port range is configured on the same page.

> The service is a read-only file listing — no write endpoints; directory contents are visible live as they change, which suits continuously produced logs and artifacts.
