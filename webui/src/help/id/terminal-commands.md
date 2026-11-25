Perintah Cepat menyematkan perintah build/test/deploy proyek ke menu klik kanan terminal — sekali klik untuk menjalankan, tersedia di mana saja. Perintah dikelola per repositori di `.vterm/commands.json` di akar repositori.

## Membuat commands.json

Klik kanan di dalam terminal → `Commands`:

- Ketika repositori sesi belum punya commands.json, menu menawarkan satu entri, `Generate commands.json` — mengekliknya membuat server mendeteksi tipe proyek (npm / Cargo / .NET / Xcode / Gradle-Android / generik) dan menulis templat awal yang konservatif. Berkas non-kosong yang ada **tidak pernah ditimpa**;
- Lokasi berkas: `.vterm/commands.json` di akar repositori Git (direktori kerja saat ini untuk folder non-Git).

## Mempertajam

Templat yang dihasilkan hanya berisi perintah lazim untuk tipe proyek yang terdeteksi. Biarkan agen kode Anda (mis. agen editor yang tersambung ke MCP server ini) membaca proyek dan mempertajam berkas:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

Array tingkat atas juga diterima; `env` opsional dan mengatur variabel sebelum perintah berjalan; kunci lama `value` masih dibaca. Kolom opsional `os` membatasi entri ke sistem operasi tempat `vterm-server` berjalan: gunakan `"win"` untuk Windows, `"linux"` untuk Linux, atau `"osx"` untuk macOS. Jika `os` dihilangkan, diatur ke `"all"`, atau berisi nilai yang tidak dikenal, entri tersedia di semua platform. Server mengawasi CWD dan perubahan commands.json dan menyiarkannya — buka ulang menu dan perintah baru sudah ada.

## Menjalankan

Klik kanan → `Commands` → klik entri untuk menjalankannya di terminal itu (env opsional diterapkan dulu). Menu tidak tersedia selama aplikasi agen berjalan di sesi, agar teks perintah tidak mendarat di UI aplikasi.

## Contoh: build & instalasi Android lewat Wi-Fi

Panduan lengkap pada proyek Gradle-Android: sematkan penyiapan ADB nirkabel dan alur build-instal ke menu klik kanan, lalu bangun ulang aplikasi dengan sekali klik.

**1. Alihkan ponsel ke ADB nirkabel.** Dua perintah ini diketik sekali per perangkat — `adb tcpip 5555` selagi ponsel di USB, lalu `adb connect` dengan alamat ponsel:

![Terminal — mengalihkan ponsel Android ke ADB nirkabel dengan adb tcpip 5555 dan adb connect](images/terminal-commands-adb-setup.gif)

**2. Biarkan agen kode menulis perintah.** Buka agen kode proyek (di sini Claude Code) di terminal dan minta menambahkan perintah build & instal — ia membaca tata letak proyek dan menulis entri ke `.vterm/commands.json`:

![Terminal — Claude Code menyunting commands.json dengan perintah build & instal APK](images/terminal-commands-agent-edit.mp4)

**3. Build dan instal dari menu.** Pisahkan terminal agar keluaran build mendapat panel sendiri, lalu klik kanan → `Commands` → entri — Gradle merakit APK dan menginstalnya ke ponsel lewat Wi-Fi:

![Terminal — memisahkan panel dan menjalankan perintah build & instal Android dari menu klik kanan](images/terminal-commands-split-run.mp4)
