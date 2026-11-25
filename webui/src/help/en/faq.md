## The browser warns “Your connection is not private” — is that normal?

Yes. Forwarded ports, the Web UI and client connections share the same self-signed server certificate; the encryption itself is not compromised. Import the server certificate into your trust store to silence it permanently.

## Is Vibe Terminal free?

Yes — the desktop client/server and headless server are free. Download and install on as many machines as you like.

## Where does my data go?

Nowhere you didn’t put it. Sessions, credentials (enc-v2:) and voice recognition all live on your machines; pairing keys never travel in plain text.

## Can I expose a forwarded service safely?

Yes. The built-in shares are token-gated: the links the client generates for a `Host with HTTP server` share or the Server Settings page carry an access token, and a connection with no valid token and no valid session is dropped before anything is served. On the first visit the token is exchanged for a secure session cookie scoped to that one service and disappears from the URL — a settings ticket works exactly once and expires within a minute, and stopping or restarting a share invalidates its earlier links and sessions. Treat the complete link as the credential and hand it only to people you trust. A forward to your own service adds TLS with the server certificate but no authentication — keep auth in front of sensitive services, and tear a forward down instantly from the Port Forwarding panel when you are done.
