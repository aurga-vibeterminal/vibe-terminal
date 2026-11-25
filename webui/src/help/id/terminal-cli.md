`vtermc` adalah klien baris perintah: ia terhubung ke server, membuat atau menautkan sesi terminal, dan mengelola daftar server tersimpan — semuanya dari dalam terminal biasa, lewat SSH, atau di mesin headless. Ia berbagi `~/.vibeterminal/client.json` dengan klien desktop, sehingga keduanya melihat server yang sama.

## Memulai terminal

Jalankan `vtermc` tanpa flag mode untuk membuat terminal baru di server lokal (`wss://localhost:5153` secara bawaan) dengan shell yang terdeteksi, dan mengalirkannya ke terminal saat ini. Ketik `exit` (atau `Ctrl+D`) di shell jarak jauh untuk mengakhiri sesi.

Sesi hidup di server: jika koneksi terputus — kehilangan jaringan, menutup jendela, membunuh `vtermc` — sesi tetap berjalan dan dapat ditautkan kembali nanti dengan `vtermc -s`.

- `vtermc -p <id>` — mulai dengan profil terminal tertentu (daftar profil ada di bagian `Profiles` Web UI);
- `vtermc -p` — tanpa id profil: pemilih menampilkan profil server secara interaktif;
- `vtermc -w <dir>` — mengatur direktori kerja sesi baru.

Ketika stdin bukan terminal (pipe atau berkas), `vtermc` beralih ke mode batch: stdin dialirkan ke sesi, dan keluaran mengalir sampai sesi ditutup — praktis dari skrip atau agen:

```sh
echo "make test" | vtermc
```

## Menghubungkan ke server

- `--server wss://host:port` — hubungkan ke server tertentu alih-alih lokal (alamat IPv6 dalam kurung siku: `wss://[fd00::1]:5153`); variabel lingkungan `VTERM_SERVER` menetapkan bawaan yang sama;
- Handshake dijawab dengan kunci rahasia server: server yang ditambahkan lewat `-a`/`-s` membawa kuncinya di `client.json`; server lokal jatuh ke variabel lingkungan `TERMINAL_GUI_SECRET_KEY` (atau kunci teks polos di `server.json`).

## Menambahkan server

`-a` menyimpan server ke `client.json` dan keluar — ia tidak pernah membuka terminal. Dua cara:

- `vtermc -a` — permintaan interaktif: `Server name`, `Server address`, dan `Secret key`. Alamat harus `wss://host:port` (masukan tidak valid diminta ulang; alamat kosong membatalkan). Nama kosong jatuh ke alamat;
- `vtermc -a "vtshare1:..."` — tempel string berbagi terenkripsi. Mengeklik `QR Code` pairing di bagian `Server` Web UI menyalinnya ke papan klip; `vtermc` mendekripsinya dan mengisi nama, alamat, dan kunci rahasia secara otomatis.

Detail yang perlu diketahui:

- menambahkan ulang alamat yang ada memperbarui entri itu di tempat (id-nya dipertahankan) alih-alih menduplikasinya;
- alamat lokal ditolak — `Local` selalu tersedia implisit dan tidak pernah butuh entri;
- entri juga ditampilkan oleh Server Manager klien desktop.

## Memilih server dan sesi

`vtermc -s` membuka dua pemilih layar penuh (memerlukan terminal interaktif):

1. **Daftar server** — `Local` lebih dulu, lalu setiap server terkelola sebagai `Name  (address)`. Navigasi dengan tombol panah atau `j`/`k`, lompat dengan angka, `Enter` memilih, `q` membatalkan;
2. **Daftar terminal** — `New Terminal` lebih dulu, diikuti sesi hidup server itu dengan direktori kerja, judul, dan server pemilik. Tautkan ke sesi yang ada, atau buat baru (yang lalu mematuhi `-p` / `-w` seperti menjalankan `vtermc` biasa).

Menautkan tidak pernah mengganggu sesi — ia terus berjalan di server di antara penautan.

### Menghapus server

Di daftar server, bergeraklah ke server terkelola dan tekan `d`: baris memerah dan petunjuk meminta konfirmasi. Tekan `y` (atau `Enter`) untuk menghapus entri dari `client.json`, tombol lain membatalkan. `Local` tidak dapat dihapus.

## Variabel lingkungan

Sesi baru mewarisi kumpulan tetap dari shell Anda — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — daftar dipisah koma (atau variabel lingkungan `VTERM_ENV_INHERIT`); wildcard `*` didukung;
- `-e VAR=VALUE` — menyuntikkan variabel tambahan (dapat diulang);
- `--no-env` — memulai sesi tanpa variabel warisan sama sekali.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Semua opsi

```text
-p, --profile [<ID>]          Pilih profil terminal (hilangkan untuk daftar interaktif)
-w, --working-dir <DIR>       Atur direktori kerja
-s, --select                  Pilih server dan terminal yang ada secara interaktif
-a, --add [<SHARE>]           Tambahkan server (vtshare1: string berbagi, atau interaktif)
    --server <URL>            URL WebSocket server [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Variabel lingkungan tambahan
    --no-env                  Jangan mewarisi variabel lingkungan
    --env-inherit <PATTERNS>  Pola variabel lingkungan untuk diwarisi [env: VTERM_ENV_INHERIT]
```
