Server memiliki host Model Context Protocol bawaan, dan bagian `MCP` pada Web UI digunakan untuk mengonfigurasinya: Code Agent (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) dapat terhubung dari chat mereka sendiri dan bekerja langsung di mesin Anda — membaca dan mengendalikan terminal, bahkan mengoperasikan EasyEDA Pro.

## Pengaturan server

Aktifkan server MCP lalu atur:

- `Port` (bawaan `6180`) dan `Bind Address` — `127.0.0.1` menjaga endpoint hanya di localhost; `0.0.0.0` membukanya ke jaringan;
- `Access Token` — wajib untuk permintaan non-localhost: pemanggil jarak jauh harus menyajikannya sebagai Bearer token, sedangkan permintaan dari localhost selalu diizinkan. Token tidak pernah kosong — instalasi baru menyertakan token acak; tombol dadu membuat ulang, tombol salin mengeluarkannya;
- `Stream Throttle` — interval minimum antar dorongan stream layar ke agent yang tersambung.

Perubahan berlaku saat `Save`; server memuat ulang tanpa restart.

## Endpoint

Setiap set kapabilitas dilayani di URL miliknya sendiri, `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — permukaan terminal bawaan. Agent dapat mendaftarkan terminal, membaca layar terlihat dan riwayat gulir, mengetik masukan mentah dan tombol bernama, mengubah ukuran, serta menunggu prompt. Panggilan alat menargetkan satu terminal lewat Terminal ID-nya — klik kanan terminal → `MCP` → `Copy Terminal ID` lalu tempel ID ke konteks agent (lihat Terminal MCP di bagian Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — addon MCP bawaan yang menjembatani EasyEDA Pro ke agent: satu set alat tetap yang kecil plus katalog panggilan `eda.*` yang dapat dicari, diteruskan ke ekstensi konektor EasyEDA lewat WebSocket. Aktifkan dulu addonnnya di bagian `Addons` — endpoint-nya muncul di daftar setelah terpasang, dan lencana menunjukkan apakah daemon addon sedang tersambung. Bidang `Timeout` addon membatasi tiap permintaan terjembatani (naikkan untuk operasi EDA yang lambat);
- **Addon MCP lainnya** — addon bertipe MCP apa pun yang terpasang menambahkan endpoint-nya dengan cara yang sama, masing-masing dengan lencana tersambung/terputus secara langsung.

Pilih CLI agent Anda di dropdown di atas daftar; tombol salin di samping endpoint kemudian memberikan perintah penyiapan yang siap dijalankan untuk aplikasi itu.

## Menghubungkan agent

Token tidak pernah perlu ditempel manual. Server memelihara profil lingkungan `Vibe Terminal MCP Bearer Token` yang dibuat otomatis (lihat bagian Environment Variables) yang mengekspor `VTERM_MCP_TOKEN`; terapkan ke terminal sebelum menghubungkan agent, lalu gunakan perintah yang disalin:

- **Aplikasi perintah shell** (Claude Code, Codex) — jalankan perintah `mcp add` yang disalin di terminal biasa. Codex membaca token dari `VTERM_MCP_TOKEN` lewat `--bearer-token-env-var`, jadi tidak ada rahasia yang berakhir di baris perintah;
- **Aplikasi percakapan** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — mulai aplikasi dengan profil diterapkan, lalu kirim prompt yang disalin ke dalam percakapannya. Agent langsung memperbarui konfigurasi MCP-nya. Kimi menggunakan `bearerTokenEnvVar`; Pi, OpenCode, dan Qwen Code menggunakan token Bearer dari `VTERM_MCP_TOKEN`, sehingga token itu sendiri tidak pernah muncul dalam chat.

Di dalam Pi dan Qwen Code, `/mcp` memeriksa koneksi.

Kartu `Command Templates` di bawah pengaturan menyimpan perintah penyiapan per aplikasi: sunting entri, timpa bawaan, atau tambah templat untuk CLI agent baru — pratinjau menunjukkan persis apa yang dihasilkan tombol salin.
