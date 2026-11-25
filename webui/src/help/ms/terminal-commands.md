Perintah Pantas menyematkan perintah build/test/deploy projek ke menu klik kanan terminal — satu klik untuk melaksanakan, tersedia di mana-mana. Perintah diselenggara setiap repositori dalam `.vterm/commands.json` di akar repositori.

## Mencipta commands.json

Klik kanan dalam terminal → `Commands`:

- Apabila repositori sesi belum ada commands.json, menu menawarkan satu entri, `Generate commands.json` — mengkliknya menjadikan pelayan mengesan jenis projek (npm / Cargo / .NET / Xcode / Gradle-Android / generik) dan menulis templat mula yang berhemah. Fail sedia ada yang tidak kosong **tidak pernah ditulis ganti**;
- Lokasi fail: `.vterm/commands.json` di akar repositori Git (direktori kerja semasa untuk folder bukan-Git).

## Mempertajam

Templat yang dijana hanya mengandungi perintah lazim bagi jenis projek yang dikesan. Biarkan ejen kod anda (cth. ejen penyunting yang bersambung ke MCP pelayan ini) membaca projek dan mempertajam fail:

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

Tatasusunan aras atas turut diterima; `env` pilihan dan menetapkan pemboleh ubah sebelum perintah berjalan; kunci lama `value` masih dibaca. Medan pilihan `os` mengehadkan entri kepada sistem pengendalian yang menjalankan `vterm-server`: gunakan `"win"` untuk Windows, `"linux"` untuk Linux, atau `"osx"` untuk macOS. Jika `os` diabaikan, ditetapkan kepada `"all"`, atau mengandungi nilai yang tidak dikenali, entri tersedia pada setiap platform. Pelayan memerhati CWD dan perubahan commands.json dan menyiarkannya — buka semula menu dan perintah baharu sudah ada.

## Melaksanakan

Klik kanan → `Commands` → klik satu entri untuk melaksanakannya dalam terminal itu (env pilihan diterapkan dahulu). Menu tidak tersedia selagi aplikasi ejen berjalan dalam sesi, supaya teks perintah tidak mendarat dalam UI aplikasi.

## Contoh: bina & pasang Android melalui Wi-Fi

Panduan penuh pada projek Gradle-Android: sematkan persediaan ADB wayarles dan aliran bina-pasang ke menu klik kanan, kemudian bina semula aplikasi dengan satu klik.

**1. Tukar telefon kepada ADB wayarles.** Dua perintah ini ditaip sekali setiap peranti — `adb tcpip 5555` selagi telefon pada USB, kemudian `adb connect` dengan alamat telefon:

![Terminal — menukar telefon Android kepada ADB wayarles dengan adb tcpip 5555 dan adb connect](images/terminal-commands-adb-setup.gif)

**2. Biarkan ejen kod menulis perintah.** Buka ejen kod projek (di sini Claude Code) dalam terminal dan minta menambah perintah bina & pasang — ia membaca susun atur projek dan menulis entri ke `.vterm/commands.json`:

![Terminal — Claude Code menyunting commands.json dengan perintah bina & pasang APK](images/terminal-commands-agent-edit.mp4)

**3. Bina dan pasang daripada menu.** Bahagikan terminal supaya output bina mendapat anak tetingkapnya sendiri, kemudian klik kanan → `Commands` → entri — Gradle memasang APK dan memasangnya ke telefon melalui Wi-Fi:

![Terminal — memecahkan anak tetingkap dan melaksanakan perintah bina & pasang Android daripada menu klik kanan](images/terminal-commands-split-run.mp4)
