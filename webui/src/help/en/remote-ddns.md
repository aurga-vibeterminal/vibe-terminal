Home and office broadband typically comes with a dynamic public IP that changes without notice. DDNS keeps a fixed domain pointed at the machine's current IP — clients and other servers only need to remember the name, never the address.

## How it works

![DDNS architecture — the manager periodically detects the local IP, the addon updates the DNS record through the provider API, clients connect by domain](diagram/ddns)

The whole pipeline runs automatically in the server background:

1. **DDNS manager** refreshes on a fixed cycle (every 5 minutes by default, adjustable via the `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS` environment variable); saving the configuration triggers an immediate refresh, and nothing runs while the service is disabled;
2. **Local IP detection** reads the selected network interface first, falling back to route-based discovery, and filters the result by the `IP Family` setting (IPv4 / IPv6 / dual);
3. **DDNS addon** — a sandboxed JS script — receives the decrypted config and the current IP and calls the DNS provider's API over HTTP. The builtin Cloudflare addon, for example, looks up the zone by domain and rewrites the A / AAAA records to the current IP, deleting stale records when an address family disappears;
4. **Clients** resolve the domain to the latest IP and connect by name.

## Configuration

Open the `DDNS` section of the Web UI:

1. Turn on `Enable`. You can also leave the service off while filling everything in — a disabled service saves its configuration but never runs;
2. Pick a provider under `Provider`; `Cloudflare DDNS` is built in;
3. Fill in the provider fields (Cloudflare example):
   - `Email` — the Cloudflare account email;
   - `API Token` — a token with Zone → DNS → Edit permission;
   - `Domain` — the zone root domain, e.g. `example.com`;
   - `Subdomain` — the record name under the zone, e.g. `sub` for `sub.example.com`; leave empty for the zone root;
4. In `Network Interface & IP`, choose the `Interface` and `IP Family`. `Dual Stack` requires the interface to have both an IPv4 and an IPv6 address;
5. Click `Save`, then `Sync Now` to verify. `Sync Status` shows the last IP pushed and the sync time, along with the addon's error message on failure.

## Security

Provider credentials (API tokens) are encrypted before being stored (`enc-v2:`) and only decrypted in memory while the addon runs; addons execute in a sandboxed JS runtime with outbound HTTP as their only capability.

## Extending providers

Providers are addons: a JSON manifest (declaring the config fields the Web UI turns into a form) plus a JS script exposing `run()`. The builtin `cloudflare-ddns` is the reference implementation — custom providers can be uploaded as a zip in the `Addons` section.

## Working with other features

- Once the domain resolves, the `Server` section's QR code can switch `Connection Type` to `DDNS Domain`, so clients scan a domain connection;
- Other servers can put `wss://your-domain:port` into their `Relay Address` to run the relay chain over the domain (see the Relay chapter).
