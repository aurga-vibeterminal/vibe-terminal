Web UI adalah halaman konfigurasi server itu sendiri: setiap fitur server diatur ke dalam sebuah bagian yang didaftar di bilah sisi kiri. Ia bukan terminal — pekerjaan sehari-hari terjadi di klien desktop atau `vtermc` (lihat Ikhtisar). Halaman ini menjelaskan tata letak dan perilaku umumnya; bab-bab fitur membahas pengaturan tiap bagian secara rinci.

## Tata Letak

- **Bilah sisi** — daftar bagian (di bawah), ikon bantuan di samping nama server, dan versi aplikasi di bagian bawah; di layar sempit ia meluncur masuk dari kiri dan tombol hamburger di header mengalihkannya;
- **Header** — judul bagian yang sedang dibuka ditambah, di kanan atas, pemilih bahasa dan tombol terang/gelap (di layar sempit ikon bantuan juga berada di sana);
- **Konten** — formulir bagian yang dipilih, terpusat dan dapat digulir.

## Bagian

Dalam urutan bilah sisi:

- `Server` — pengaturan port, sertifikat TLS, dan autentikasi: `Secret Key`, pairing `QR Code`, alamat relay, dan sakelar `Auto-start on Login` semuanya berada di sini (lihat Menghubungkan Klien dan Panduan Penyiapan). Di Linux, petunjuk mulai-otomatis juga menjelaskan login otomatis display manager untuk mencapai desktop sebelum login lokal (lihat Penyiapan Server di Desktop Jarak Jauh & Simulator);
- `Terminal` — analisis terminal: deteksi prompt, konfirmasi otomatis, dan klasifikasi LLM (lihat Konfirmasi Otomatis);
- `UART` — profil terminal serial berbasis port UART lokal (lihat Serial UART);
- `Add-on` — add-on terpasang: aktifkan, sunting, atau unggah (penyedia LLM adalah add-on — lihat LLM);
- `LLM` — menetapkan penyedia LLM untuk setiap fitur AI (lihat LLM);
- `Suara` — penyedia pengenalan suara, model, dan pengaturan transkripsi (lihat Penyiapan Server di Input Suara);
- `Variabel Lingkungan` — koleksi variabel lingkungan dalam pohon grup (lihat Variabel Lingkungan);
- `MCP` — server MCP terminal yang diekspos ke agen kode (lihat Terminal MCP);
- `DDNS` — penyedia DDNS dan pengaturan sinkronisasi (lihat DDNS);
- `Pemajuan Port` — kumpulan port terusan dan penerusan TLS yang aktif (lihat Port Forwarding);
- `Static HTTP` — folder yang dihosting lewat HTTP dan berbagi berkas yang aktif (lihat Publikasikan dengan Server HTTP);
- `Desktop Jarak Jauh` — pengaturan penangkapan, masukan, dan aliran desktop jarak jauh (lihat Penyiapan Server di Desktop Jarak Jauh & Simulator);
- `Statistik LLM` — penggunaan token, jumlah permintaan, dan metrik kinerja di seluruh fitur LLM.

## Menyimpan & tips

- Sebagian besar bagian menyimpan otomatis tak lama setelah penyuntingan — indikator menyimpan dan setiap kesalahan muncul di bagian bawahnya;
- `MCP`, `DDNS`, dan `Desktop Jarak Jauh` menahan suntingan sampai `Simpan Perubahan` ditekan: selama suntingan tertunda, petunjuk `Unsaved changes` muncul di samping tombol, dan `Atur Ulang` membuangnya;
- Dibuka dari klien seluler, Web UI mengikuti bahasa dan tema aplikasi serta menyembunyikan kontrol bahasa dan temanya sendiri.
