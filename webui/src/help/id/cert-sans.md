Sertifikat leaf hanya sah untuk nama-nama yang tercantum dalam **Nama Alternatif Subjeknya (SAN)**. Ketika klien terhubung, nama host atau IP di URL-nya harus sama persis dengan salah satu entri itu — tidak ada dukungan wildcard dan tidak ada pencocokan awalan. Halaman ini membahas pengelolaan daftar itu di sisi server; pemeriksaan di sisi klien ada di halaman terakhir, Verifikasi Ketat di Klien.

## Deteksi otomatis

Server terus-menerus mendeteksi nama-nama yang membuatnya dapat dijangkau — nama host saat ini dan alamat antarmuka jaringan yang aktif — dan mencantumkannya di bawah `Terdeteksi otomatis`. Entri ini mengikuti mesin dan tidak bisa diedit manual: tambahkan antarmuka jaringan atau ubah nama host, daftar akan menyesuaikan pada pembaruan berikutnya.

## Domain dan alamat IP tambahan

Deteksi otomatis tidak mengetahui semuanya: nama DDNS, entri DNS split-horizon, atau alamat di balik NAT perlu dicantumkan manual. Tambahkan di bawah `Domain dan alamat IP tambahan`:

1. Ketik nama host atau alamat IP (misalnya `server.home.arpa` atau `10.0.0.8`) lalu tekan `Tambah`;
2. Ulangi untuk setiap nama yang akan dipakai klien. Hapus entri dengan tombol tong sampah;
3. Periksa `Pratinjau SAN akhir` — tampilan gabungan entri otomatis dan manual yang akan dibawa sertifikat berikutnya;
4. Tekan `Simpan & Terbitkan Ulang`. Menyimpan menandatangani sertifikat leaf baru 90 hari dengan daftar SAN baru dan memuatnya ulang secara langsung — koneksi yang ada tetap berjalan, dan handshake baru langsung memakai sertifikat baru.

## Penerbitan ulang manual dan status

Panel `Sertifikat Leaf Server` menampilkan sertifikat yang aktif: lencana status, subjek, sidik jari SHA-256, rentang keberlakuan, dan entri SAN. Lencana status berarti:

- `Tersedia` — sertifikat leaf yang sah telah dimuat dan sedang digunakan;
- `Pembaruan gagal` — upaya penerbitan ulang terakhir gagal; sertifikat sebelumnya tetap aktif dan kesalahan ditampilkan di halaman;
- `Belum dibuat` — belum ada CA maupun sertifikat leaf; gunakan `Buat CA & Sertifikat` untuk membuat keduanya.

`Terbitkan Ulang Sertifikat Leaf` langsung menandatangani sertifikat leaf baru tanpa menunggu pemicu otomatis — berguna tepat setelah mengubah nama host atau antarmuka, atau untuk menerapkan pengeditan SAN yang dilakukan di luar Web UI.

## Mendistribusikan CA akar

Klien yang akan memakai verifikasi ketat membutuhkan sertifikat publik CA akar. Dapatkan dari kartu `CA Akar Privat`:

- `Unduh CA Akar` menyimpan berkas PEM;
- `Salin CA Akar` menaruh PEM di papan klip, siap ditempel ke kolom CA klien.

CA akar tidak pernah berubah sepanjang pembaruan sertifikat leaf maupun pengeditan SAN — mendistribusikannya satu kali sudah cukup. Sebelum mengaktifkan verifikasi ketat di mana pun, bandingkan sidik jari SHA-256 sertifikat yang ditampilkan di halaman ini dengan yang diterima klien (lihat halaman terakhir, Verifikasi Ketat di Klien).

> Unduhan dan papan klip hanya berisi sertifikat publik CA — tidak pernah kunci privat. Serahkan hanya kepada orang-orang yang Anda ingin mampu memverifikasi server Anda.
