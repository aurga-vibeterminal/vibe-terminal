Relay memecahkan satu masalah: kebanyakan mesin berada di belakang NAT atau IP dinamis, sehingga klien tak dapat menelepon masing-masing langsung. Biarkan satu mesin yang dapat dijangkau publik menjadi titik masuk relay; setiap mesin lain menelepon keluar kepadanya. Klien lalu memelihara satu koneksi ke relay dan menjangkau terminal tiap server berbagi melaluinya.

## Arsitektur

![Arsitektur relay — tiap klien menjaga satu koneksi ke relay; server berbagi menelepon relay ke hulu](diagram/relay)

- **Server relay** — titik masuk publik tempat klien dan server berbagi terhubung. Biarkan `Relay Address`-nya kosong.
- **Server berbagi** — mesin yang `Relay Address`-nya menunjuk ke relay. Ia menelepon ke hulu dan membagikan terminalnya melaluinya. Garis warna pada tiap simpul berbagi di diagram mencerminkan garis yang dilukis klien desktop pada tab server itu — satu warna per server.
- Koneksi selalu berasal dari dalam: server berbagi menelepon relay, jadi tak perlu penerusan port atau IP publik pada mereka.

## Apa yang dilihat klien

Sekali terhubung ke relay, tiap server berbagi berperilaku seperti mesin lokal:

- Tab terminal membawa garis warna dan ikon OS jarak jauh, mengidentifikasi mesin sekali lirik (lihat Panduan Penyiapan);
- menu profil mengelompokkan terminal per server — buka terminal di mesin mana pun langsung;
- grup tab dapat dilepas ke jendelinya sendiri, mengabdikan satu jendela untuk satu mesin.

## Model keamanan

Klien dan server berbagi dipasangkan dengan kunci rahasia yang sama; koneksi terowong melalui relay. Kunci berjalan dalam QR code pairing / string berbagi, tak pernah teks polos.

Instruksi langkah demi langkah ada di halaman berikut, Panduan Penyiapan.
