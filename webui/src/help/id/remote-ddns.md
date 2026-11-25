Broadband rumah dan kantor umumnya datang dengan IP publik dinamis yang berubah tanpa kabar. DDNS menjaga domain tetap mengarah ke IP mesin saat ini — klien dan server lain hanya perlu mengingat nama, bukan alamat.

## Cara kerjanya

![Arsitektur DDNS — manajer mendeteksi IP lokal berkala, addon memperbarui catatan DNS lewat API penyedia, klien terhubung lewat domain](diagram/ddns)

Seluruh pipeline berjalan otomatis di latar belakang server:

1. **Manajer DDNS** menyegarkan pada siklus tetap (tiap 5 menit bawaan, dapat diatur lewat variabel lingkungan `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); menyimpan konfigurasi memicu penyegaran segera, dan tak ada yang berjalan selama layanan nonaktif;
2. **Deteksi IP lokal** membaca antarmuka jaringan terpilih lebih dulu, jatuh ke penemuan berbasis rute, dan menyaring hasilnya menurut pengaturan `IP Family` (IPv4 / IPv6 / ganda);
3. **Addon DDNS** — skrip JS bersandbox — menerima konfigurasi terdekripsi dan IP saat ini lalu memanggil API penyedia DNS lewat HTTP. Addon Cloudflare bawaan misalnya mencari zona menurut domain dan menulis ulang catatan A / AAAA ke IP saat ini, menghapus catatan basi ketika satu keluarga alamat hilang;
4. **Klien** me-resolve domain ke IP terbaru dan terhubung dengan nama.

## Konfigurasi

Buka bagian `DDNS` Web UI:

1. Nyalakan `Enable`. Anda juga dapat membiarkan layanan mati sambil mengisi semuanya — layanan nonaktif menyimpan konfigurasinya tapi tak pernah berjalan;
2. pilih penyedia di bawah `Provider`; `Cloudflare DDNS` terpasang bawaan;
3. isi bidang penyedia (contoh Cloudflare):
   - `Email` — surel akun Cloudflare;
   - `API Token` — token dengan izin Zone → DNS → Edit;
   - `Domain` — domain akar zona, mis. `example.com`;
   - `Subdomain` — nama catatan dalam zona, mis. `sub` untuk `sub.example.com`; biarkan kosong untuk akar zona;
4. di `Network Interface & IP`, pilih `Interface` dan `IP Family`. `Dual Stack` mensyaratkan antarmuka punya alamat IPv4 dan IPv6;
5. klik `Save`, lalu `Sync Now` untuk memverifikasi. `Sync Status` menampilkan IP terakhir yang didorong dan waktu sinkron, beserta pesan galat addon saat gagal.

## Keamanan

Kredensial penyedia (token API) dienkripsi sebelum disimpan (`enc-v2:`) dan hanya didekripsi di memori selama addon berjalan; addon dieksekusi dalam runtime JS bersandbox dengan HTTP keluar sebagai satu-satunya kemampuannya.

## Memperluas penyedia

Penyedia adalah addon: manifest JSON (mendeklarasikan bidang konfigurasi yang diubah Web UI menjadi formulir) plus skrip JS yang mengekspos `run()`. `cloudflare-ddns` bawaan adalah implementasi referensi — penyedia kustom dapat diunggah sebagai zip di bagian `Addons`.

## Bekerja dengan fitur lain

- begitu domain ter-resolve, QR code bagian `Server` dapat mengalihkan `Connection Type` ke `DDNS Domain`, sehingga klien memindai koneksi domain;
- server lain dapat menaruh `wss://your-domain:port` di `Relay Address` mereka untuk menjalankan rantai relay lewat domain (lihat bab Relay).
