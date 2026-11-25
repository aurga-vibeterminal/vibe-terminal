import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { readFileSync } from 'node:fs';
import { fileURLToPath, URL } from 'node:url';

// Single source of truth for the version the sidebar displays: the Rust
// workspace's [workspace.package] version in ../Cargo.toml. Injected as
// __APP_VERSION__ at build time so the UI always matches the binaries
// built from the same checkout (they carry env!("CARGO_PKG_VERSION")).
function workspaceVersion(): string {
  const cargoToml = fileURLToPath(new URL('../Cargo.toml', import.meta.url));
  const cargo = readFileSync(cargoToml, 'utf8');
  const section = cargo.split(/^\[workspace\.package\]\s*$/m)[1] ?? '';
  const version = section.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) {
    throw new Error(`No [workspace.package] version found in ${cargoToml}`);
  }
  return version;
}

// `npm run dev` serves the UI standalone. /api requests always use
// server-relative URLs, so without a named backend they simply hit the dev
// server origin — the same-origin behaviour the Rust server provides when
// it embeds this UI in production. To tunnel /api (HTTP + WebSocket) to a
// running backend, name it:
//
//   npm run dev --backend=8701                  -> http://127.0.0.1:8701
//   npm run dev -- -- --backend 8701            (see npm note below)
//   npm run dev -- --backend http://192.168.1.20:5180
//   VT_BACKEND=8701 npm run dev                 (env var fallback)
//
// A bare number (or :number) is expanded to http://127.0.0.1:PORT, a bare
// host:port to http://host:PORT; a full http(s):// URL is used as-is.
// `secure: false` accepts self-signed certs. Note: `npm run dev --
// --backend 8701` makes npm forward `--backend` to the vite CLI, which
// rejects unknown options — use `--backend=8701` or a double `--`.
function normalizeApiTarget(value: string): string {
  if (/^https?:\/\//i.test(value)) return value;
  if (/^\d+$/.test(value)) return `http://127.0.0.1:${value}`;
  if (/^[\w.-]+:\d+$/.test(value)) return `http://${value}`;
  return `http://127.0.0.1:${value.replace(/^:/, '')}`;
}

/** Empty string = no backend requested -> no proxy, same-origin /api. */
function resolveApiTarget(argv: string[]): string {
  const flag = argv.find((arg) => arg === '--backend' || arg.startsWith('--backend='));
  const raw =
    flag === '--backend'
      ? argv[argv.indexOf(flag) + 1]
      : flag?.slice('--backend='.length);
  const value = (raw ?? process.env.npm_config_backend ?? process.env.VT_BACKEND ?? '').trim();
  return value ? normalizeApiTarget(value) : '';
}

const apiTarget = resolveApiTarget(process.argv);
if (apiTarget) {
  console.log(`[vite] proxying /api (HTTP + WS) -> ${apiTarget}`);
}

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(workspaceVersion()),
  },
    build: {
        rollupOptions: {
            output: {
                entryFileNames: '[name].js',
                chunkFileNames: '[name].js',
                assetFileNames: '[name].[ext]'
            }
        }
    },
  server: apiTarget
    ? {
        proxy: {
          '/api': {
            target: apiTarget,
            changeOrigin: true,
            ws: true,
            secure: false,
          },
        },
      }
    : undefined,
});
