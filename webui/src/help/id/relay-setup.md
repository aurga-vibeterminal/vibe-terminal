Menghubungkan mesin ke relay memakan dua bidang: alamat relay dan kuncinya. Lihat halaman sebelumnya, Ikhtisar, untuk apa fitur ini dan bagaimana potongan-potongannya cocok.

## Menyiapkan server relay

1. Buka bagian `Server` di mesin yang akan menjadi relay.
2. Pastikan `Secret Key` diatur — tombol dadu membuat satu. Kunci yang sama memasangkan klien dan server berbagi.
3. Buka blok `QR Code` dan pilih `Network Interface` / `IP Version` yang harus dipakai klien dan server berbagi untuk menjangkaunya.

## Menghubungkan server berbagi

Detail koneksi relay berjalan di papan klip — tanpa mengetik ulang:

1. Di Web UI server relay, klik `QR code`. Ini menyalin string berbagi terenkripsi (alamat, port, kunci rahasia) ke papan klip.
2. Buka Web UI di mesin yang akan dibagikan, bagian `Server`, dan klik ke bidang `Relay Address`.
3. Peramban memunculkan dialog menanyakan izin situs ini mengakses papan klip — klik `Allow`:

![Peramban — dialog izin akses papan klip saat bidang difokuskan](images/relay-allow-paste.jpg)

4. Dengan izin diberikan, papan klip diperiksa: saat berisi string berbagi, tombol tempel muncul di samping bidang — klik untuk mendekripsi dan mengisi `Relay Address` dan `Relay Secret Key` otomatis:

![Web UI — alamat relay dan kunci rahasia relay terisi otomatis dari string berbagi yang ditempel](images/relay-paste-share.jpg)

5. Tinjau nilainya dan klik `Apply` — menyimpan menyambung ulang relay hulu.

![Web UI — bidang relay terisi dengan tombol Apply](images/relay-apply-share.jpg)

> Membaca papan klip memerlukan konteks aman (HTTPS atau localhost). Pada HTTP polos tombol tempel tetap tersembunyi; ketik alamat dan kunci relay secara manual.

Secara bawaan, koneksi hulu dienkripsi tetapi tidak diverifikasi. Untuk juga mengunci sertifikat relay, lihat Verifikasi Ketat pada Relay di bagian Sertifikat & TLS.

## Klien desktop

### Tab server berbagi

Tab terminal yang berjalan di server berbagi membawa dua penanda: garis warna di tepi kiri tab — satu warna per server — dan ikon OS jarak jauh (Windows / Apple / Linux) di samping titik status. Tab lokal di server relay sendiri tanpa penanda.

![Klien desktop — penanda tab terminal server berbagi](images/relay-tab-indicator.jpg)

### Profil terminal per server

Menu profil (bilah judul) mengelompokkan profil terminal menurut server: profil server relay yang tersambung lebih dulu, lalu satu grup terlipat per server berbagi, dinamai dari mesinnya. Buka grup dan pilih profil untuk membuka terminal di server itu. Menu mengikuti grup tab yang dipilih di bilah judul.

![Klien desktop — menu profil dengan grup per server](images/relay-profile-menu.jpg)

### Pengelompokan dan jendela terpisah

Pemilih grup bilah judul mendaftar grup tab semua server; grup bawaan server berbagi tampil sebagai `Ungrouped (Nama Server)`. Submenu tiap grup mendaftar tabnya untuk berpindah cepat dan menawarkan `Move Group to New Window` yang melepaskan seluruh grup ke jendelinya sendiri — berguna untuk mengabdikan jendela atau monitor bagi satu mesin.

![Klien desktop — menu pemilih grup dengan aksi pindah-ke-jendela-baru](images/relay-group-selector.jpg)
