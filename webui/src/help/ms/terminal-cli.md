`vtermc` ialah klien baris perintah: ia bersambung ke pelayan, mencipta atau menyambung sesi terminal dan mengurus senarai pelayan tersimpan — semuanya dalam terminal biasa, melalui SSH, atau pada mesin headless. Ia berkongsi `~/.vibeterminal/client.json` dengan klien desktop, jadi kedua-duanya melihat pelayan yang sama.

## Memulakan terminal

Jalankan `vtermc` tanpa bendera mod untuk mencipta terminal baharu pada pelayan tempatan (`wss://localhost:5153` secara lalai) dengan shell yang dikesan, dan menstrimkannya ke terminal semasa. Taip `exit` (atau `Ctrl+D`) dalam shell jauh untuk mengakhiri sesi.

Sesi hidup di pelayan: jika sambungan terputus — kehilangan rangkaian, menutup tetingkap, membunuh `vtermc` — sesi terus berjalan dan boleh disambung semula kemudian dengan `vtermc -s`.

- `vtermc -p <id>` — mulakan dengan profil terminal tertentu (senarai profil ada di bahagian `Profiles` Web UI);
- `vtermc -p` — tanpa id profil: pemilih menyenaraikan profil pelayan secara interaktif;
- `vtermc -w <dir>` — tetapkan direktori kerja sesi baharu.

Apabila stdin bukan terminal (paip atau fail), `vtermc` bertukar ke mod kelompok: stdin disalurkan ke sesi, dan output mengalir sehingga sesi ditutup — berguna daripada skrip atau ejen:

```sh
echo "make test" | vtermc
```

## Menyambung ke pelayan

- `--server wss://host:port` — sambung ke pelayan tertentu dan bukannya tempatan (alamat IPv6 dalam kurungan: `wss://[fd00::1]:5153`); pemboleh ubah persekitaran `VTERM_SERVER` menetapkan lalai yang sama;
- Jabat tangan dijawab dengan kunci rahsia pelayan: pelayan yang ditambah melalui `-a`/`-s` membawa kuncinya dalam `client.json`; pelayan tempatan jatuh kepada pemboleh ubah persekitaran `TERMINAL_GUI_SECRET_KEY` (atau kunci teks jelas dalam `server.json`).

## Menambah pelayan

`-a` menyimpan pelayan ke `client.json` dan keluar — ia tidak pernah membuka terminal. Dua cara:

- `vtermc -a` — gesaan interaktif: `Server name`, `Server address` dan `Secret key`. Alamat mesti `wss://host:port` (input tidak sah diminta semula; alamat kosong membatalkan). Nama kosong jatuh kepada alamat;
- `vtermc -a "vtshare1:..."` — tampal rentetan perkongsian tersulit. Mengklik `QR Code` pemasangan di bahagian `Server` Web UI menyalinnya ke papan klip; `vtermc` menyahsulitkannya dan mengisi nama, alamat dan kunci rahsia secara automatik.

Butiran yang patut diketahui:

- menambah semula alamat sedia ada mengemas kini entri itu di tempat (idnya dikekalkan) dan bukannya menduplikasinya;
- alamat tempatan ditolak — `Local` sentiasa tersedia secara tersirat dan tidak pernah memerlukan entri;
- entri turut dipaparkan oleh Server Manager klien desktop.

## Memilih pelayan dan sesi

`vtermc -s` membuka dua pemilih skrin penuh (memerlukan terminal interaktif):

1. **Senarai pelayan** — `Local` dahulu, kemudian setiap pelayan terurus sebagai `Name  (address)`. Navigasi dengan kekunci anak panah atau `j`/`k`, lompat dengan digit, `Enter` memilih, `q` membatalkan;
2. **Senarai terminal** — `New Terminal` dahulu, diikuti sesi hidup pelayan itu dengan direktori kerja, tajuk dan pelayan pemilik. Sambung ke sesi sedia ada, atau cipta yang baharu (yang kemudiannya mematuhi `-p` / `-w` seperti pelaksanaan `vtermc` biasa).

Penyambungan tidak pernah mengganggu sesi — ia terus berjalan di pelayan antara penyambungan.

### Membuang pelayan

Dalam senarai pelayan, gerak ke pelayan terurus dan tekan `d`: baris menjadi merah dan petunjuk meminta pengesahan. Tekan `y` (atau `Enter`) untuk memadam entri daripada `client.json`, sebarang kekunci lain membatalkan. `Local` tidak boleh dipadam.

## Pemboleh ubah persekitaran

Sesi baharu mewarisi set tetap daripada shell anda — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — senarai dipisahkan koma (atau pemboleh ubah persekitaran `VTERM_ENV_INHERIT`); kad bebas `*` disokong;
- `-e VAR=VALUE` — menyuntik pemboleh ubah tambahan (boleh diulang);
- `--no-env` — memulakan sesi tanpa sebarang pemboleh ubah diwarisi.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Semua pilihan

```text
-p, --profile [<ID>]          Pilih profil terminal (tinggalkan untuk senarai interaktif)
-w, --working-dir <DIR>       Tetapkan direktori kerja
-s, --select                  Pilih pelayan dan terminal sedia ada secara interaktif
-a, --add [<SHARE>]           Tambah pelayan (vtshare1: rentetan perkongsian, atau interaktif)
    --server <URL>            URL WebSocket pelayan [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Pemboleh ubah persekitaran tambahan
    --no-env                  Jangan warisi pemboleh ubah persekitaran
    --env-inherit <PATTERNS>  Corak pemboleh ubah persekitaran untuk diwarisi [env: VTERM_ENV_INHERIT]
```
