## Peramban memperingatkan “Koneksi Anda tidak pribadi” — apakah itu wajar?

Wajar. Port terusan, Web UI, dan koneksi klien memakai sertifikat self-signed server yang sama; enkripsinya sendiri tak terganggu. Impor sertifikat server ke penyimpanan kepercayaan perangkat untuk membungkam peringatan itu secara permanen.

## Apakah Vibe Terminal gratis?

Ya — klien/server desktop dan server headless gratis. Unduh dan pasang di sebanyak mesin yang Anda suka.

## Ke mana data saya pergi?

Tidak ke mana pun selain tempat Anda meletakkannya. Sesi, kredensial (enc-v2:), dan pengenalan suara semuanya hidup di mesin Anda; kunci pairing tak pernah berjalan dalam teks polos.

## Bisakah saya mengekspos layanan terusan dengan aman?

Bisa. Berbagi bawaan memerlukan token: tautan yang dibuat klien untuk berbagi `Host with HTTP server` maupun halaman Settings menyertakan token akses, dan koneksi tanpa token yang valid dan tanpa sesi yang valid diputus sebelum apa pun disajikan. Pada kunjungan pertama, token ditukar dengan cookie sesi aman yang hanya berlaku untuk layanan tersebut dan hilang dari URL — ticket Settings hanya berlaku satu kali dan kedaluwarsa dalam semenit, sedangkan menghentikan atau memulai ulang layanan mematikan tautan dan sesi sebelumnya. Anggap tautan lengkap sebagai kredensial dan berikan hanya kepada orang yang Anda percayai. Penerusan ke layanan Anda sendiri menambah TLS dengan sertifikat server tetapi tanpa autentikasi — pertahankan autentikasi di depan layanan sensitif, dan bongkar penerusan seketika dari panel penerusan port begitu selesai.
