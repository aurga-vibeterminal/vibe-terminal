Secara bawaan, server berbagi mengenkripsi koneksi hulunya ke relay tetapi tidak memverifikasi siapa di ujung satunya — sikap percaya pada pemakaian pertama yang sama seperti menerima sertifikat yang ditandatangani sendiri di peramban. **Strict Relay TLS** mengubah hal itu: server berbagi memeriksa sertifikat relay terhadap CA akar yang Anda berikan dan menolak terhubung kecuali rantai, keberlakuan, dan nama host semuanya lolos. Ini membuat serangan man-in-the-middle terhadap tautan relay praktis mustahil, dengan imbalan satu langkah manual: memberi server berbagi CA akar milik relay. Halaman ini menerapkan ide yang sama dengan halaman sebelumnya, Verifikasi Ketat di Klien, pada tautan relay hulu milik server berbagi.

## Di mana mengaturnya

Verifikasi ketat dikonfigurasi pada server berbagi — mesin yang `Relay Address`-nya menunjuk ke relay. Di Web UI mesin tersebut, buka bagian `Server`: di bawah `Relay Secret Key` ada baris `Strict Relay TLS` yang menampilkan status `On` / `Off` saat ini. Perluas untuk melihat:

1. `Upstream Relay Root CA` — kolom multibaris untuk sertifikat publik CA akar milik relay;
2. Sakelar `Off` / `On` — `On` baru dapat dipilih setelah PEM terbaca sebagai sertifikat yang valid.

![Web UI — blok Strict Relay TLS yang diperluas di bagian Server: kolom Upstream Relay Root CA dengan sakelar Off/On dan tombol Apply](images/cert-relay-strict-tls.jpg)

Pengaturan relay tidak pernah disimpan di tengah penyuntingan: tinjau nilainya lalu klik `Apply`. Menyimpan langsung menyambungkan ulang relay hulu dengan pengaturan baru.

## Membawa CA akar ke server berbagi

Dua cara, dari sumber yang sama (Web UI relay itu sendiri):

- **Menempel string berbagi** — klik `QR code` di Web UI server relay. String berbagi menyertakan CA akar relay bersama alamat dan kunci, sehingga menempelkannya ke kolom `Relay Address` milik server berbagi juga mengisi kolom CA secara otomatis (lihat Panduan Penyiapan di bagian Relay). Mode ketat tetap mati sampai Anda meninjau CA dan mengaktifkannya sendiri;
- **Salin & tempel** — tekan `Salin CA Akar` di halaman `Sertifikat & TLS` milik relay dan tempel PEM ke kolom `Upstream Relay Root CA`.

CA akar sama untuk setiap server berbagi dan tidak pernah berubah sepanjang pembaruan sertifikat — mengaturnya satu kali per relay sudah cukup.

Sebelum mempercayai CA akar, pastikan Anda mendapatkan yang asli: halaman `Sertifikat & TLS` milik relay menampilkan sidik jari SHA-256 milik CA akar. Bandingkan dengan PEM yang akan Anda tempel. Jika berbeda, jangan aktifkan verifikasi ketat — dapatkan CA melalui kanal terpercaya sebagai gantinya.

## Apa saja yang diperiksa mode ketat

Dengan sakelar pada `On`, koneksi `RelayPrimary` hulu harus menampilkan sertifikat yang:

1. Merantai ke CA akar yang dikonfigurasi — sertifikat yang ditandatangani oleh pihak lain apa pun, termasuk root sistem publik, ditolak;
2. Masih dalam rentang keberlakuannya — sertifikat kedaluwarsa atau belum berlaku ditolak;
3. Membawa SAN yang cocok dengan host dari `Relay Address` yang Anda tuju — sertifikat yang diterbitkan untuk nama lain ditolak.

Satu batasan: hanya koneksi relay hulu yang diverifikasi. Koneksi balik Terminal dan permintaan tetap memakai kebijakan sertifikat yang ada.

Jika ada pemeriksaan yang gagal, koneksi ditolak dan dicoba ulang di latar belakang dengan jeda yang meningkat; server berbagi tetap tidak dapat dijangkau melalui relay sampai pemeriksaan lolos atau Anda mematikan mode ketat.

## Kegagalan yang umum

- `Verifikasi TLS ketat memerlukan PEM CA akar` — sakelar menyala tetapi kolom CA kosong. Tempel PEM dulu;
- `SAN tidak cocok` — Anda memanggil relay dengan nama atau IP yang tidak ada dalam sertifikatnya. Tambahkan nama itu di halaman `Sertifikat & TLS` milik relay (lihat halaman Pengelolaan Domain & SAN) lalu terbitkan ulang;
- `Sertifikat kedaluwarsa atau belum berlaku` — jam relay salah, atau pembaruan sertifikat leaf gagal. Periksa status sertifikat pada relay;
- `Tidak ditandatangani CA yang dikonfigurasi` — CA di kolom milik relay lain, atau koneksi sedang disadap. Bandingkan sidik jari sebelum melanjutkan.
