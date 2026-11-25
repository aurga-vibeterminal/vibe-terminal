Menyambungkan mesin ke relay mengambil dua medan: alamat relay dan kuncinya. Lihat halaman sebelumnya, Gambaran Keseluruhan, untuk apa fitur ini dan bagaimana bahagiannya serasi.

## Menyiapkan pelayan relay

1. Buka bahagian `Server` pada mesin yang akan bertindak sebagai relay.
2. Pastikan `Secret Key` ditetapkan — butang dadu menjana satu. Kunci yang sama memasangkan klien dan pelayan dikongsi.
3. Buka blok `QR Code` dan pilih `Network Interface` / `IP Version` yang patut digunakan klien dan pelayan dikongsi untuk mencapainya.

## Menyambungkan pelayan dikongsi

Butiran sambungan relay bergerak di papan klip — tanpa menaip semula:

1. Pada Web UI pelayan relay, klik `QR code`. Ini menyalin rentetan perkongsian tersulit (alamat, port, kunci rahsia) ke papan klip.
2. Buka Web UI pada mesin yang akan dikongsi, bahagian `Server`, dan klik ke medan `Relay Address`.
3. Pelayar memaparkan dialog bertanya sama ada membenarkan laman ini mengakses papan klip — klik `Allow`:

![Pelayar — dialog kebenaran akses papan klip dipaparkan apabila medan difokuskan](images/relay-allow-paste.jpg)

4. Dengan akses diberikan, papan klip disemak: apabila ia mengandungi rentetan perkongsian, butang tampal muncul di sebelah medan — klik untuk menyahsulit dan mengisi `Relay Address` dan `Relay Secret Key` secara automatik:

![Web UI — alamat relay dan kunci rahsia relay diisi automatik daripada rentetan perkongsian yang ditampal](images/relay-paste-share.jpg)

5. Semak nilainya dan klik `Apply` — menyimpan menyambung semula relay hulu.

![Web UI — medan relay terisi dengan butang Apply](images/relay-apply-share.jpg)

> Membaca papan klip memerlukan konteks selamat (HTTPS atau localhost). Pada HTTP tulen butang tampal kekal tersembunyi; taip alamat dan kunci relay secara manual.

Secara lalai, sambungan hulu disulitkan tetapi tidak disahkan. Untuk turut mengepit sijil relay, lihat Pengesahan Ketat pada Relay di bahagian Sijil & TLS.

## Klien desktop

### Tab pelayan dikongsi

Tab terminal yang berjalan pada pelayan dikongsi membawa dua penanda: jalur berwarna pada tepi kiri tab — satu warna setiap pelayan — dan ikon OS jauh (Windows / Apple / Linux) di sebelah titik status. Tab tempatan pada pelayan relay itu sendiri tiada penanda.

![Klien desktop — penanda tab terminal pelayan dikongsi](images/relay-tab-indicator.jpg)

### Profil terminal setiap pelayan

Menu profil (bar tajuk) mengumpulkan profil terminal mengikut pelayan: profil pelayan relay yang bersambung itu dahulu, kemudian satu kumpulan boleh lipat setiap pelayan dikongsi, dinamakan sempena mesinnya. Buka kumpulan dan pilih profil untuk membuka terminal pada pelayan itu. Menu mengikut kumpulan tab yang dipilih dalam bar tajuk.

![Klien desktop — menu profil dengan kumpulan setiap pelayan](images/relay-profile-menu.jpg)

### Pengumpulan dan tetingkap berasingan

Pemilih kumpulan bar tajuk menyenaraikan kumpulan tab semua pelayan; kumpulan lalai pelayan dikongsi dipaparkan sebagai `Ungrouped (Nama Pelayan)`. Submenu setiap kumpulan menyenaraikan tabnya untuk pertukaran pantas dan menawarkan `Move Group to New Window` yang merenggangkan keseluruhan kumpulan ke tetingkapnya sendiri — berguna untuk menauliahkan satu tetingkap atau monitor bagi satu mesin.

![Klien desktop — menu pemilih kumpulan dengan tindakan pindah-ke-tetingkap-baharu](images/relay-group-selector.jpg)
