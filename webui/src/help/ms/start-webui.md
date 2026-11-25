Web UI ialah halaman konfigurasi pelayan itu sendiri: setiap ciri pelayan disusun kepada bahagian, disenaraikan dalam bar sisi kiri. Ia bukan terminal — kerja harian berlaku dalam klien desktop atau `vtermc` (lihat Gambaran Keseluruhan). Halaman ini menerangkan susun atur dan kelakuan biasa; bab-bab ciri menerangkan tetapan setiap bahagian secara terperinci.

## Susun Atur

- **Bar Sisi** — senarai bahagian (di bawah), ikon bantuan di sebelah nama pelayan, dan versi aplikasi di bahagian bawah; pada skrin sempit ia meluncur masuk dari kiri dan butang hamburger pengepala menogolkannya;
- **Pengepala** — tajuk bahagian semasa ditambah, di penjuru kanan atas, pemilih bahasa dan togol cerah/gelap (pada skrin sempit ikon bantuan juga berada di sana);
- **Kandungan** — borang bahagian yang dipilih, berpusat dan boleh skrol.

## Bahagian

Mengikut susunan bar sisi:

- `Pelayan` — tetapan port, sijil TLS dan pengesahihan: `Secret Key`, `QR Code` pemasangan, alamat relay dan togol mula-auto semuanya berada di sini (lihat Menyambungkan Klien dan Panduan Persediaan). Pada Linux, petua mula-auto turut menerangkan daftar masuk automatik pengurus paparan untuk mencapai desktop sebelum daftar masuk tempatan (lihat Persediaan Pelayan dalam Desktop Jauh & Simulator);
- `Terminal` — analisis terminal: pengesanan gesaan, pengesahan automatik dan pengelasan LLM (lihat Pengesahan Automatik);
- `UART` — profil terminal siri yang disokong oleh port UART tempatan (lihat Siri UART);
- `Tambahan` — addon yang dipasang: aktifkan, sunting atau muat naiknya (penyedia LLM adalah addon — lihat LLM);
- `LLM` — menugaskan penyedia LLM kepada setiap ciri AI (lihat LLM);
- `Suara` — tetapan penyedia pengecaman suara, model dan transkripsi (lihat Persediaan Pelayan dalam Input Suara);
- `Pemboleh Ubah Persekitaran` — koleksi pemboleh ubah persekitaran dalam pokok kumpulan (lihat Pemboleh Ubah Persekitaran);
- `MCP` — pelayan MCP terminal yang didedahkan kepada ejen kod (lihat Terminal MCP);
- `DDNS` — tetapan penyedia dan penyegerakan DDNS (lihat DDNS);
- `Pemajuan Port` — kumpulan port yang dimajukan dan pemajuan TLS yang aktif (lihat Pemajuan Port);
- `Static HTTP` — direktori yang dihoskan melalui HTTP dan perkongsian fail yang aktif (lihat Terbitkan dengan Pelayan HTTP);
- `Desktop Jauh` — tetapan tangkapan, input dan strim untuk desktop jauh (lihat Persediaan Pelayan dalam Desktop Jauh & Simulator);
- `Statistik LLM` — penggunaan token, bilangan permintaan dan metrik prestasi merentas semua ciri LLM.

## Penyimpanan & petua

- Kebanyakan bahagian menyimpan secara automatik sebentar selepas suntingan — penunjuk penyimpanan dan sebarang ralat muncul di bahagian bawah bahagian;
- `MCP`, `DDNS` dan `Desktop Jauh` menahan suntingan sehingga `Simpan Perubahan` ditekan: selagi suntingan belum disimpan, petunjuk `Unsaved changes` muncul di sebelah butang, dan `Tetapkan Semula` membuangnya;
- Dibuka daripada klien mudah alih, Web UI mengikut bahasa dan tema aplikasi dan menyembunyikan kawalan bahasa dan tema sendirinya.
