The leaf certificate is only valid for the names listed in its **Subject Alternative Names (SAN)**. When a client connects, the hostname or IP in its URL must exactly match one of those entries — there is no wildcard support and no prefix matching. This page covers managing that list on the server; for checking it on the client side, see the last page, Strict Verification on Clients.

## Automatic discovery

The server continuously discovers the names it is reachable by — current hostnames and the addresses of active network interfaces — and lists them under `Automatically discovered`. These entries track the machine and cannot be edited by hand: add a network interface or change the hostname and the list follows on the next renewal.

## Additional domains and IP addresses

Discovery does not know about everything: DDNS names, split-horizon DNS entries, or an address behind NAT all need to be listed by hand. Add them under `Additional domains and IP addresses`:

1. Type a hostname or IP address (for example `server.home.arpa` or `10.0.0.8`) and press `Add`;
2. Repeat for every name clients will use. Remove an entry with the trash button;
3. Check the `Final SAN preview` — the merged view of discovered and manual entries that the next certificate will carry;
4. Press `Save & Reissue`. Saving signs a new 90-day leaf with the new SAN list and hot-reloads it — existing connections stay up, and new handshakes use the new certificate right away.

## Manual reissue and status

The `Server Leaf Certificate` panel shows the live certificate: a status badge, subject, SHA-256 fingerprint, validity window, and SAN entries. The status badge reads:

- `Available` — a valid leaf is loaded and in use;
- `Update failed` — the last reissue attempt failed; the previous certificate remains active and the error is shown on the page;
- `Not generated` — no CA or leaf exists yet; use `Generate CA & Certificate` to create both.

`Reissue Leaf Certificate` signs a fresh leaf immediately without waiting for an automatic trigger — useful right after changing the hostname or interfaces, or to pick up SAN edits made outside the Web UI.

## Distributing the Root CA

Clients that will use strict verification need the Root CA's public certificate. Get it from the `Private Root CA` card:

- `Download Root CA` saves the PEM file;
- `Copy Root CA` puts the PEM on the clipboard, ready to paste into a client's CA field.

The CA never changes across leaf renewals or SAN edits — distributing it once is enough. Before enabling strict verification anywhere, compare the certificate's SHA-256 fingerprint on this page against what the client received (see the last page, Strict Verification on Clients).

> The download and clipboard contain the CA's public certificate only — never a private key. Only hand the CA to people you want to be able to verify your server.
