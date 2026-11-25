Server melindungi semua titik akhir TLS yang dieksposnya — Web UI, koneksi klien, dan port yang diteruskan — dengan sertifikat yang diterbitkan oleh **CA akar privat** miliknya sendiri. Halaman `Sertifikat & TLS` di Web UI adalah tempat Anda mengelola CA tersebut, mengontrol nama-nama yang sah untuk sertifikat, dan menyerahkan CA kepada klien yang ingin memverifikasi server secara ketat.

## Cara kerjanya

- Server membuat CA akar privat hanya satu kali dan menyimpannya sepanjang umur instalasi. Semuanya ditandatangani olehnya;
- CA menandatangani **sertifikat leaf** untuk server dengan masa berlaku 90 hari, mencakup nama host dan alamat IP yang benar-benar dipakai klien untuk mengaksesnya;
- Ketika nama host, antarmuka jaringan, atau entri SAN mesin berubah, sertifikat leaf diterbitkan ulang secara otomatis — CA akar tidak pernah berubah, sehingga klien yang sudah mempercayainya tetap mempercayai server;
- Sertifikat leaf baru ditukar melalui resolver sertifikat bersama dan dimuat ulang secara langsung (hot-reload): koneksi yang ada tidak pernah diputus, dan handshake baru langsung memakai sertifikat terbaru. Jika pembaruan gagal, sertifikat sebelumnya tetap aktif.

![Halaman Sertifikat & TLS: di atas kartu CA Akar Privat, di tengah pengelolaan SAN, di bawah detail sertifikat leaf server](images/certificates-page.jpg)

## Yang Anda temukan di halaman ini

1. `CA Akar Privat` — unduh atau salin sertifikat publik CA untuk dibagikan ke klien (dan perambannya). Unduhan tidak pernah berisi kunci privat;
2. `Domain & Alamat IP (SAN)` — daftar nama yang sah untuk sertifikat leaf. Dijelaskan di halaman berikutnya, Pengelolaan Domain & SAN;
3. `Sertifikat Leaf Server` — sertifikat yang sedang aktif: status, subjek, sidik jari SHA-256, rentang keberlakuan, dan entri SAN.

## Sertifikat TLS kustom

Jika sertifikat TLS kustom dikonfigurasi untuk server, sertifikat itu tetap yang berlaku: sertifikat terkelola di halaman ini tetap dibuat dan dijaga mutakhirnya, tetapi tidak menggantikan identitas kustom yang telah dikonfigurasi. Selama identitas kustom aktif, halaman menampilkan pemberitahuan.

## Mengapa mempercayai CA akar

Klien yang hanya butuh enkripsi langsung berfungsi tanpa pengaturan. Klien yang ingin *memverifikasi* server — mode ketat yang dijelaskan di halaman terakhir, Verifikasi Ketat di Klien — membutuhkan sertifikat publik CA akar, dan itu harus melewati kanal yang Anda percayai:

> Unduh CA akar hanya dari Web UI lokal server ini atau melalui tunnel terpercaya, dan verifikasi sidik jari SHA-256 (ditampilkan di halaman ini) sebelum mengaktifkan verifikasi ketat di mana pun.

CA yang sama juga menandatangani sertifikat yang dipakai penerusan port (lihat halaman Penerusan Port). Setelah sebuah perangkat mempercayai CA akar, URL yang diteruskan terbuka tanpa peringatan peramban *koneksi Anda tidak privat*.
