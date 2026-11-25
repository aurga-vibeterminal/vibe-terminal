Port forwarding mempublikasikan layanan HTTP yang berjalan di mesin server — server dev, dasbor perangkat, apa pun yang mendengarkan di alamat `http://localhost:port` polos — pada port TLS-nya sendiri, sehingga perangkat mana pun yang dapat menjangkau server membukanya di peramban sebagai `https://<alamat-server>:<port>/`.

## Cara kerjanya

- Sebuah penerusan memetakan **port target** — layanan HTTP di mesin ini — ke **port terusan** yang dialokasikan dari rentang port terkonfigurasi (bawaan `62000`–`62999`);
- Port terusan berbicara TLS: seluruh lalu lintas dienkripsi dengan **sertifikat self-signed** server — tak ada yang berjalan dalam teks polos, bahkan lewat jaringan tak tepercaya;
- Perangkat mana pun yang dapat menjangkau server — di LAN, atau dari internet begitu rentang port diteruskan di router — membuka `https://<alamat-server>:<portTerusan>/` dan tiba di layanan lokal.

Penerusan dibuat sesuai permintaan: berbagi berkas `Host with HTTP server` di server jarak jauh (terhubung lewat relay) otomatis meminta satu penerusan dan menyerahkan URL yang siap dibuka kepada Anda. Tak perlu penyiapan manual.

## Mengelola penerusan

![Web UI — bagian Port Forwarding: konfigurasi rentang port dan daftar penerusan yang aktif](images/port-forwarding-webui.jpg)

1. Buka bagian `Pemajuan Port` di Web UI server.
2. `Port Range Configuration` membatasi kumpulan tempat port terusan dialokasikan; perubahan tersimpan otomatis. Persempit rentangnya bila firewall atau router Anda hanya meneruskan beberapa port.
3. Daftar aktif menampilkan tiap penerusan sebagai `:portTerusan → :portTarget` beserta jumlah koneksi aktifnya.
4. Tombol henti sebuah penerusan mematikan listener-nya seketika dan memutus koneksi aktifnya; berbagi berikutnya mengalokasikan port baru. `Refresh` membaca ulang daftar.

## Peringatan sertifikat

Penerusan dienkripsi dengan sertifikat self-signed milik server sendiri — sertifikat yang sama yang melindungi Web UI dan koneksi klien. Peramban tak dapat memverifikasinya terhadap otoritas sertifikat, jadi kunjungan pertama ke URL terusan menampilkan peringatan privasi (*Koneksi Anda tidak pribadi*). Ini wajar dan aman untuk dilewati: enkripsinya sendiri tak terganggu, hanya asal sertifikatnya yang tak dapat diverifikasi mesin. Untuk membungkam peringatan itu secara permanen, impor sertifikat server ke penyimpanan kepercayaan perangkat Anda.

![Peramban — peringatan sertifikat self-signed tak tepercaya; lanjutkan lewat Advanced → Proceed untuk membuka halaman](images/port-forwarding-cert-warning.jpg)

## Chrome di macOS menampilkan `ERR_ADDRESS_UNREACHABLE`

Pada macOS 15 (Sequoia) dan lebih baru, app pihak ketiga memerlukan izin eksplisit untuk menjangkau perangkat di jaringan lokal — alamat `192.168.x.x`, `10.x.x.x`, atau sejenisnya. Safari dikecualikan sebagai app sistem, sehingga URL terusan dapat terbuka baik di Safari sementara Chrome melaporkan `ERR_ADDRESS_UNREACHABLE` bahkan sebelum TLS dimulai. Ini bukan masalah sertifikat dan bukan masalah pada penerusan itu sendiri.

Untuk memberi Chrome akses:

1. Buka `Pengaturan Sistem` → `Privasi & Keamanan` → `Jaringan Lokal`.
2. Temukan `Google Chrome` dalam daftar dan **nyalakan**.
3. Keluar sepenuhnya dari Chrome (`Cmd+Q`, bukan sekadar menutup jendela) lalu luncurkan ulang.

Jika saklarnya sudah menyala, matikan lalu nyalakan lagi — ia dapat macet setelah pembaruan Chrome atau macOS. Jika galat berlanjut, periksa ekstensi VPN atau proksi di Chrome yang tidak merutekan rentang alamat privat.

## Kontrol akses

Apa yang melindungi port terusan bergantung pada apa yang ada di baliknya:

- **Berbagi bawaan memerlukan token.** Tautan berbagi yang dibuat klien — berbagi `Host with HTTP server` (`?share_token=…`) dan halaman Settings (`?ticket=…`) — menyertakan token akses. Koneksi tanpa token yang valid dan tanpa sesi yang valid diputus sebelum apa pun disajikan. Pada kunjungan pertama, peramban menukarnya dengan cookie sesi aman yang hanya berlaku untuk layanan tersebut, dan token hilang dari URL — tak tersisa di bilah alamat maupun riwayat.

- **Token sekali pakai dan berumur pendek.** Ticket Settings hanya berlaku satu kali dan kedaluwarsa dalam semenit; saat layanan berhenti atau dimulai ulang, tautan dan sesi sebelumnya berhenti berfungsi. Anggap tautan lengkap sebagai kredensial — siapa pun yang memilikinya dapat mengakses layanan — dan bagikan hanya kepada orang yang Anda percayai.

- **Layanan Anda sendiri membawa autentikasinya sendiri.** Penerusan ke layanan lokal lain mengenkripsi lalu lintas dengan sertifikat yang ditandatangani sendiri oleh server, tetapi tidak menambahkan autentikasi: apa pun yang disajikan layanan tersebut — halaman maupun API — dapat dijangkau semua orang yang bisa menjangkau port. Pasang autentikasi pada layanan sensitif sebelum mengeksposnya.
