Manajemen berkas memungkinkan Anda bekerja dengan berkas di server langsung dari klien: bilah aktivitas kiri memiliki dua entri — `Recent` (proyek terbaru) dan penjelajah berkas — dipakai berdampingan dengan tab terminal. Semuanya beroperasi pada server sesi: mesin lokal, atau mesin jarak jauh/berbagi lewat relay.

## Recent: proyek terbaru

`Recent` mendaftar folder yang paling baru dibuka agen kode Anda di server saat ini, terbaru lebih dulu:

- **Klik** satu entri dan penjelajah berkas beralih ke direktori itu;
- **Klik kanan** membuka menu aksi cepat:
  - `Open in VS Code` — membuka direktori di VS Code di mesin ini;
  - `Reveal in File Explorer` — menampilkannya di pengelola berkas OS;
  - `Copy Full Path` / `Copy Relative Path` — jalur lengkap, atau relatif ke repositori;
  - `Search in Folder` — mencari berkas di dalam direktori itu;
  - `Open in Terminal` — membuka tab terminal baru di sana; `cd Here` — cd terminal saat ini ke sana;
  - `Host with HTTP server` — mempublikasikan direktori sebagai layanan berkas HTTP (lihat Publikasikan dengan Server HTTP);
  - `Open in App` — membukanya dengan aplikasi agen yang terdeteksi di server.

![Panel kiri — daftar proyek terbaru](images/files-overview.jpg)

## Penjelajah berkas

Tab `Explorer` menelusuri dan mengelola sistem berkas sisi server: navigasi direktori, tampilan pohon dan datar, pencarian, buat/ganti nama/hapus dan operasi berkas lain, plus unggah dan unduh seret-lepas antara server dan mesin ini. Lihat Penjelajah Berkas dan Unggah & Unduh.
