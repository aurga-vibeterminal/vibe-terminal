Hubungkan klien ke server ini dengan QR code pairing dari Web UI — atau dengan mengetik detailnya secara manual. Semua yang dibutuhkan klien (alamat, port, kunci rahasia) ditampilkan di bagian Server.

## Menyiapkan server

- Buka bagian `Server` dan pastikan `Secret Key` sudah diatur; klik tombol dadu untuk membuatnya. Membuat ulang kunci membatalkan setiap klien yang dipasangkan, yang harus memasangkan ulang.
- Buka blok `QR Code` dan pilih `Network Interface` dan `IP Version` (IPv4 / IPv6) yang harus digunakan mesin lain untuk menjangkau server ini.
- Jika domain DDNS dikonfigurasi, Anda dapat mengalihkan `Connection Type` ke `DDNS Domain` agar QR code menyematkan domain alih-alih alamat IP.
- `Pairing Preview` menampilkan URL koneksi persis (`wss://host:port`) yang akan dihubungi klien.
- Mengeklik `QR code` menyalin string berbagi terenkripsi ke papan klip — klien menggunakannya untuk isi-otomatis sekali sentuh.

## Klien desktop

- Klik tombol nama server di bilah judul dan pilih `Servers` di menu untuk membuka dialog Server Manager.

![Klien desktop — menu server yang dibuka dengan mengeklik tombol nama server](images/start-connect-server-menu.jpg)

- Klik tombol `+` di samping daftar server untuk memulai formulir Add Server.
- Jika string berbagi ada di papan klip Anda, ikon papan klip muncul di kanan atas formulir — klik untuk mendekripsi string dan mengisi `Title`, `Address`, dan `Secret Key` secara otomatis.
- Jika tidak, isi bidang secara manual: nama tampilan, URL koneksi (`wss://host:port`, alamat IPv6 dalam kurung siku), dan kunci rahasia.
- Klik `Save`, lalu pilih server baru untuk membuka sesi.

![Klien desktop — Server Manager dengan formulir Add Server](images/start-connect-add-server.jpg)
