Sambungkan klien ke pelayan ini dengan kod QR pemasangan daripada Web UI — atau dengan menaip butirannya secara manual. Semua yang diperlukan klien (alamat, port, kunci rahsia) dipaparkan di bahagian Server.

## Menyiapkan pelayan

- Buka bahagian `Server` dan pastikan `Secret Key` ditetapkan; klik butang dadu untuk menjana satu. Menjana semula kunci membatalkan setiap klien yang dipasangkan, yang perlu memasangkan semula.
- Buka blok `QR Code` dan pilih `Network Interface` dan `IP Version` (IPv4 / IPv6) yang patut digunakan mesin lain untuk mencapai pelayan ini.
- Jika domain DDNS dikonfigurasikan, anda boleh menukar `Connection Type` kepada `DDNS Domain` supaya kod QR memuatkan domain dan bukannya alamat IP.
- `Pairing Preview` menunjukkan URL sambungan yang tepat (`wss://host:port`) yang akan dail oleh klien.
- Mengklik `QR code` menyalin rentetan perkongsian tersulit ke papan klip — klien menggunakannya untuk isi-auto satu sentuhan.

## Klien desktop

- Klik butang nama pelayan di bar tajuk dan pilih `Servers` dalam menu untuk membuka dialog Server Manager.

![Klien desktop — menu pelayan dibuka dengan mengklik butang nama pelayan](images/start-connect-server-menu.jpg)

- Klik butang `+` di sebelah senarai pelayan untuk memulakan borang Add Server.
- Jika rentetan perkongsian berada di papan klip anda, ikon papan klip muncul di kanan atas borang — klik untuk menyahsulitkan rentetan dan mengisi `Title`, `Address` dan `Secret Key` secara automatik.
- Jika tidak, isikan medan secara manual: nama paparan, URL sambungan (`wss://host:port`, alamat IPv6 dalam kurungan segi empat) dan kunci rahsia.
- Klik `Save`, kemudian pilih pelayan baharu untuk membuka sesi.

![Klien desktop — Server Manager dengan borang Add Server](images/start-connect-add-server.jpg)
