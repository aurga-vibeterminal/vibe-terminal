`Host with HTTP server` mempublikasikan direktori apa pun di server sebagai layanan berkas HTTP sekali klik: perangkat lain di jaringan yang sama (atau lewat relay) — ponsel, komputer lain, rig uji — dapat menelusuri dan mengunduh direktori itu dari peramban. Ideal untuk berbagi artefak build dan bundel log, atau menyerahkan berkas ke perangkat tanpa klien terpasang.

## Cara memakainya

Tiga pintu masuk, semuanya pada direktori: menu klik kanan daftar berkas, menu klik kanan entri Recent, dan menu klik kanan breadcrumb → `Host with HTTP server`.

Mengkliknya:

1. Server memulai layanan HTTP statis berakar pada direktori itu, dengan port diambil dari rentang port Static HTTP;
2. **Server lokal**: URL `http://127.0.0.1:port/` langsung disalin ke papan klip, dengan pesan status "HTTP server: … (disalin ke papan klip)";
3. **Server jarak jauh** (terhubung lewat relay): klien otomatis meminta pemetaan proxy terbalik dan menyalin URL berbagi yang dapat dijangkau dari luar — pembatasan LAN bukan penghalang.

Buka URL itu di peramban perangkat mana pun dan direktori ditelusuri serta diunduh seperti daftar web biasa.

## Mengelola layanan berjalan

Tiap layanan HTTP terus berjalan sampai dihentikan:

- Buka bagian `Static HTTP` di Web UI; daftar `Services` menampilkan setiap layanan berjalan dan akar dokumennya;
- tiap entri punya tombol henti; rentang port dikonfigurasi di halaman yang sama.

> Layanan adalah daftar berkas hanya-baca — tanpa endpoint tulis; isi direktori terlihat langsung seiring berubah, cocok untuk log dan artefak yang terus dihasilkan.
