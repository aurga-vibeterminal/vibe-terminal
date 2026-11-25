Vibe Terminal adalah hub terminal yang dikelola dari jarak jauh untuk pengembangan embedded dan server. Satu **server** berjalan di setiap mesin yang Anda kerjakan — mesin dev, server build, papan embedded — dan **klien** terhubung kepadanya dari aplikasi desktop atau baris perintah. Panduan ini mencakup kedua sisi: Web UI yang mengonfigurasi server, dan klien yang bekerja dengannya.

## Bagaimana semuanya terhubung

- Semuanya berjalan di **server** — sesi terminal berada di sana, sehingga menutup klien, kehilangan jaringan, atau mengatur ulang jendela tidak pernah menginterupsi perintah yang sedang berjalan; sambungkan kembali dan lanjutkan dari titik terakhir;
- Klien menelepon satu koneksi WebSocket terenkripsi (`wss://host:port`; `wss://localhost:5153` untuk server lokal) dan mengautentikasi dengan **kunci rahasia** server; pairing menyerahkan kunci itu melalui QR code atau string berbagi terenkripsi (lihat Menghubungkan Klien);
- **Web UI** adalah halaman konfigurasi server itu sendiri — profil, addon, penyedia LLM, relay, penerusan port, dan lainnya. Ia bukan terminal: pekerjaan sehari-hari terjadi di klien desktop atau `vtermc`.

## Komponen

- **Server** — berjalan di mesin target; menghosting sesi terminal dan menyajikan berkas, Git, desktop jarak jauh, fitur suara, dan AI kepada klien yang terhubung;
- **Klien desktop** — ruang kerja native: terminal dengan split, grup tab, dan jendela terlepas, penjelajah berkas dan editor, alat Git, input suara, riwayat percakapan agen kode, desktop jarak jauh, dan debugging jarak jauh;
- **CLI `vtermc`** — memulai dan menautkan sesi dari terminal biasa mana pun, lewat SSH, atau di mesin headless; berbagi daftar servernya dengan klien desktop (lihat Klien Baris Perintah);
- **Web UI** — pengaturan berbasis browser untuk server itu sendiri, diatur ke dalam bagian-bagian di bilah sisi kiri (lihat Menggunakan Web UI).

## Ikon baki

Pada mesin dengan sesi desktop, server menampilkan ikon baki selama berjalan:

![Menu ikon baki — `Open` membawa klien yang berjalan ke depan atau meluncurkannya, `Settings` membuka Web UI, dan item suara memulai atau menghentikan server ASR](images/start-tray-menu.jpg)

- **Open** — membawa klien desktop yang sedang berjalan ke depan, atau meluncurkannya bila belum ada yang berjalan (mengklik ganda ikonnya melakukan hal yang sama);
- **Settings** — membuka Web UI server di browser (lihat Menggunakan Web UI);
- **Start/Stop Vibe Voice** — memulai atau menghentikan server ASR tanpa membuka Web UI, mencerminkan tombol Mulai/Hentikan di bagian `Suara`; muncul ketika pengenalan suara diaktifkan, dan labelnya mengikuti status (mis. `Stop Vibe Voice (model on device)` saat model termuat);
- **Exit** — mematikan server.

Selama folder dihosting lewat HTTP, entri `HTTP Servers` menawarkan penghentian sekali klik untuk setiap berbagi (lihat Publikasikan dengan Server HTTP).

## Peta fitur

Sisa panduan ini mencerminkan bab-bab pada bilah sisi:

- **Terminal** — membuat, membagi, dan mengorganisasikan terminal; profil, WSL, dan serial UART; perintah cepat dan variabel lingkungan; mengendalikan terminal dari agen eksternal dengan MCP; meninjau percakapan agen kode; konfirmasi otomatis untuk prompt interaktif;
- **Berkas** — proyek terbaru, penjelajah berkas sisi server, unggah & unduh, dan mempublikasikan direktori lewat HTTP (lihat Penjelajah Berkas, Unggah & Unduh, dan Publikasikan dengan Server HTTP);
- **Git** — staging, commit berbantuan AI, worktree, rebase interaktif, penyelesaian konflik, dan push;
- **Desktop Jarak Jauh & Simulator** — menonton dan mengendalikan layar server, perangkat dan emulator Android, serta simulator iOS;
- **Input Suara** — pengenalan ucapan sisi server dengan dikte push-to-talk dan polesan teks oleh LLM;
- **Relay** — menjangkau mesin di belakang NAT atau IP dinamis melalui satu titik masuk publik;
- **Akses Jarak Jauh** — domain DDNS dan penerusan port TLS untuk layanan yang berjalan di server;
- **Fitur AI** — penyedia LLM sebagai addon, ditetapkan per fitur, plus integrasi MCP (lihat LLM dan MCP).

## Mulai dari sini

1. Buka bagian `Server` di Web UI, atur `Secret Key`, dan pasangkan klien — Menghubungkan Klien memandu penyiapan lewat QR code maupun manual;
2. Telusuri bagian-bagian Web UI untuk mengonfigurasi profil, addon, dan fitur — lihat Menggunakan Web UI;
3. Buka terminal di server dan kerjakan bab-bab di atas sesuai kebutuhan.
