Penjelajah berkas mengoperasikan sistem berkas sisi server. Bilah alat di atas panel kiri berpindah antara Recent / Explorer / Git / Debug / Search; Explorer adalah peramban berkasnya.

## Tombol Home: lompat ke cwd terminal

Klik tombol `Home` (rumah) di bilah alat:

- Beralih ke tampilan `Explorer`;
- Lompat ke **direktori kerja terminal aktif** (cwd-nya);
- Menyematkan direktori itu sebagai pohon dan mengosongkan kotak cari — Anda selalu mulai dari "di mana pun terminal berada, itulah yang Anda lihat".

![Penjelajah berkas mode normal — Home, breadcrumb, dan daftar datar](images/files-explorer.jpg)

## Mode pohon dan pencarian

- **Sematkan sebagai pohon**: tombol pin di ujung kanan breadcrumb beralih antara daftar datar dan mode pohon. Di mode pohon folder meluas menjadi pohon direktori yang dapat dilipat (lompatan Home menyemat otomatis);
- **Cari**: kotak cari di atas menyaring daftar saat ini berdasar nama sambil mengetik; `Search in Folder` dari menu konteks Recent menjalankan pencarian rekursif atas seluruh direktori.

![Penjelajah berkas mode pohon — akar tersemat meluas menjadi pohon direktori terlipat](images/files-explorer-tree.jpg)

## Operasi dasar

Klik kanan berkas atau folder:

- **Buat**: `New Folder` / `New File`;
- **Sunting**: `Rename`; `Copy` / `Cut` / `Paste` (bekerja antar direktori);
- **Hapus**: `Delete`, dengan dialog konfirmasi;
- **Kaitan terminal**: `Open in Terminal` membuka tab terminal baru di direktori itu; `cd to Directory` melakukan cd terminal saat ini ke sana;
- **Kaitan OS**: `Reveal in File Explorer` menampilkan direktori lokal di pengelola berkas sistem;
- **Jalur**: `Copy Full Path` / `Copy Relative Path`;
- **Publikasi**: `Host with HTTP server` mempublikasikan direktori sebagai layanan berkas HTTP (lihat Publikasikan dengan Server HTTP); `Open in App` membukanya dengan aplikasi agen yang terdeteksi di server.

Setiap operasi bekerja pada sistem berkas sisi server; sesi jarak jauh berperilaku persis seperti lokal.
