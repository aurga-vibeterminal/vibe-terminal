Secara bawaan, klien mengenkripsi setiap koneksi tetapi tidak memverifikasi siapa di ujung satunya — sikap percaya pada pemakaian pertama yang sama seperti menerima sertifikat yang ditandatangani sendiri di peramban. **Verifikasi ketat** mengubah hal itu: klien memeriksa sertifikat server terhadap CA akar yang Anda berikan dan menolak terhubung kecuali rantai, keberlakuan, dan SAN semuanya lolos. Ini membuat serangan man-in-the-middle terhadap koneksi praktis mustahil, dengan imbalan satu langkah manual: memberi klien CA akar milik server.

## Di mana mengaturnya

Verifikasi ketat dikonfigurasi per server di panel penyuntingan server milik klien, bersebelahan dengan alamat dan kunci rahasia:

1. `Sertifikat CA Akar (PEM)` — kolom multibaris untuk sertifikat publik CA akar milik server;
2. `Verifikasi CA ketat (rantai, validitas & SAN)` — hanya dapat dipilih setelah CA diisi.

![Panel penyuntingan server di klien desktop: kolom multibaris Sertifikat CA Akar (PEM) dengan kotak centang verifikasi ketat di bawahnya](images/client-ca-fields.jpg)

## Membawa CA akar ke klien

Dua cara, dari sumber yang sama (halaman `Sertifikat & TLS` milik server — lihat halaman Pengelolaan Domain & SAN):

- **Salin & tempel** — tekan `Salin CA Akar` di Web UI dan tempel PEM ke kolom `Sertifikat CA Akar (PEM)` milik klien;
- **Impor string berbagi** — string berbagi yang dihasilkan klien menyertakan CA akar bersama alamat dan kunci. Mengimpor berbagi semacam itu mengisi kolom CA secara otomatis; penerima hanya perlu mencentang kotak verifikasi ketat.

CA akar sama untuk setiap klien dan tidak pernah berubah sepanjang pembaruan sertifikat — mengaturnya satu kali per server sudah cukup.

## Apa saja yang diperiksa mode ketat

Dengan kotak centang aktif, setiap koneksi ke server itu harus menampilkan sertifikat yang:

1. Merantai ke CA akar yang dikonfigurasi — sertifikat yang ditandatangani oleh pihak lain apa pun ditolak;
2. Masih dalam rentang keberlakuannya — sertifikat kedaluwarsa atau belum berlaku ditolak;
3. Membawa SAN yang cocok dengan alamat yang Anda tuju — sertifikat yang diterbitkan untuk nama lain ditolak.

Jika ada pemeriksaan yang gagal, koneksi ditolak dengan kesalahan yang menjelaskan pemeriksaan mana yang gagal. Tanpa kotak centang (atau tanpa CA), koneksi tetap terenkripsi tetapi pemeriksaan-pemeriksaan ini dilewati.

## Memverifikasi sidik jari

Sebelum mempercayai CA akar, pastikan Anda mendapatkan yang asli: halaman `Sertifikat & TLS` menampilkan sidik jari SHA-256 milik CA akar. Bandingkan dengan PEM yang akan Anda tempel (atau PEM yang tersemat dalam berbagi yang diterima). Jika berbeda, jangan aktifkan verifikasi ketat — dapatkan CA melalui kanal terpercaya sebagai gantinya.

## Kegagalan yang umum

- `SAN tidak cocok` — Anda terhubung menggunakan nama atau IP yang tidak ada dalam sertifikat. Tambahkan di server (lihat halaman Pengelolaan Domain & SAN) lalu terbitkan ulang;
- `Sertifikat kedaluwarsa atau belum berlaku` — jam server salah, atau pembaruan sertifikat leaf gagal. Periksa status sertifikat di Web UI;
- `Tidak ditandatangani CA yang dikonfigurasi` — CA di kolom milik server lain, atau koneksi sedang disadap. Bandingkan sidik jari sebelum melanjutkan.
