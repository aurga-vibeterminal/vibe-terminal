`vtermc` is the command-line client: it connects to a server, creates or attaches to terminal sessions and manages the saved server list — all from inside a plain terminal, over SSH, or on a headless machine. It shares `~/.vibeterminal/client.json` with the desktop client, so both see the same servers.

## Starting a terminal

Run `vtermc` with no mode flags to create a fresh terminal on the local server (`wss://localhost:5153` by default) using the detected shell, and stream it into the current terminal. Type `exit` (or `Ctrl+D`) in the remote shell to end the session.

Sessions live on the server: if the connection drops — network loss, closing the window, killing `vtermc` — the session keeps running and can be re-attached later with `vtermc -s`.

- `vtermc -p <id>` — start with a specific terminal profile (the profile list is in the Web UI's `Profiles` section);
- `vtermc -p` — no profile id: the picker lists the server's profiles interactively;
- `vtermc -w <dir>` — set the working directory of the new session.

When stdin is not a terminal (a pipe or file), `vtermc` switches to batch mode: stdin is fed to the session, and output streams until the session closes — handy from scripts or agents:

```sh
echo "make test" | vtermc
```

## Connecting to a server

- `--server wss://host:port` — connect to a specific server instead of the local one (IPv6 addresses in brackets: `wss://[fd00::1]:5153`); the `VTERM_SERVER` environment variable sets the same default;
- The handshake is answered with the server's secret key: servers added through `-a`/`-s` carry their key in `client.json`; the local server falls back to the `TERMINAL_GUI_SECRET_KEY` environment variable (or a plaintext key in `server.json`).

## Adding servers

`-a` saves a server to `client.json` and exits — it never opens a terminal. Two ways:

- `vtermc -a` — interactive prompts: `Server name`, `Server address` and `Secret key`. The address must be `wss://host:port` (invalid input is re-prompted; an empty address cancels). An empty name falls back to the address;
- `vtermc -a "vtshare1:..."` — paste the encrypted share string. Clicking the pairing `QR Code` in the Web UI's `Server` section copies it to the clipboard; `vtermc` decrypts it and fills in name, address and secret key automatically.

Details worth knowing:

- re-adding an existing address updates that entry in place (its id is kept) instead of duplicating it;
- local addresses are refused — `Local` is always available implicitly and never needs an entry;
- the entries are also shown by the desktop client's Server Manager.

## Selecting servers and sessions

`vtermc -s` opens two full-screen pickers (requires an interactive terminal):

1. **Server list** — `Local` first, then every managed server as `Name  (address)`. Navigate with the arrow keys or `j`/`k`, jump with a digit, `Enter` selects, `q` cancels;
2. **Terminal list** — `New Terminal` first, followed by that server's live sessions with working directory, title and owning server. Attach to an existing session, or create a new one (which then honors `-p` / `-w` like a plain `vtermc` run).

Attaching never interrupts the session — it keeps running on the server between attaches.

### Removing a server

In the server list, move to a managed server and press `d`: the row turns red and the hint asks for confirmation. Press `y` (or `Enter`) to delete the entry from `client.json`, any other key to abort. `Local` cannot be deleted.

## Environment variables

A new session inherits a fixed set from your shell — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — comma-separated list (or the `VTERM_ENV_INHERIT` environment variable); `*` wildcards are supported;
- `-e VAR=VALUE` — inject extra variables (repeatable);
- `--no-env` — start the session with no inherited variables at all.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## All options

```text
-p, --profile [<ID>]          Select terminal profile (omit to list interactively)
-w, --working-dir <DIR>       Set working directory
-s, --select                  Select server and existing terminal interactively
-a, --add [<SHARE>]           Add a server (vtshare1: share string, or interactive)
    --server <URL>            Server WebSocket URL [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Additional environment variables
    --no-env                  Don't inherit environment variables
    --env-inherit <PATTERNS>  Environment variable patterns to inherit [env: VTERM_ENV_INHERIT]
```
