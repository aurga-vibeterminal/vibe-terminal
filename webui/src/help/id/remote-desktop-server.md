Semuanya berada di bagian `Remote Desktop` Web UI; halaman menampilkan pengaturan yang berlaku untuk platform server.

## Sakelar utama

`Enable Remote Desktop` — sekali aktif, klien jarak jauh yang tersambung boleh melihat desktop mesin ini dan mengirim masukan.

## Pengaturan platform

**Windows**

- `Windows Service` — mendaftarkan server layar sebagai layanan sistem mulai-otomatis agar desktop jarak jauh bertahan melewati reboot tanpa sesi login. `Install` / `Uninstall` memicu permintaan UAC di server (tombol menampilkan kemajuan sambil menunggu); lencana status membedakan berjalan / terpasang / tidak terpasang.
- `Mesin headless (tanpa monitor)` — tanpa tampilan tersambung (atau terjaga), Windows tak menyimpan desktop yang dapat ditangkap. Halaman menautkan unduhan penggerak Virtual Display Driver sumber terbuka (ARM64 / x64 dipilih otomatis); pasang agar desktop virtual selalu ada.

![Web UI — bagian Remote Desktop di Windows (kartu layanan dan headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — otomatis membuka kunci sesi terkunci saat aliran dimulai (bila lingkungan mendukung);
- penangkapan layar lewat Portal; `Reset Permission` pada kartu `Screen Recording Permission` menghapus persetujuan Portal tersimpan agar Anda dapat memilih layar lain.
- `Auto-start on Login` (di bagian `Server`) — pada host dengan display manager, server mulai bersama sesi grafis, sehingga penangkapan mewarisi lingkungan desktop penuh; pada host headless ia mulai saat boot sebelum login apa pun. Untuk mencapai desktop saat belum ada yang login secara lokal, aktifkan juga login otomatis di display manager (GDM, SDDM, …); demi keamanan, kunci layar otomatis boleh tetap menyala — sesi dan server terus berjalan saat terkunci, dan `Unlock on Connect` (atau `loginctl unlock-session`) membuka sesi dari jarak jauh.
![Web UI — kartu Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — sistem mensyaratkan persetujuan eksplisit untuk penangkapan. Klik `Grant Permission` untuk membuka dialog sistem; `Reset Permission` pulih dari keadaan macet.

## Alat streaming layar (dukungan Android)

`Android SDK directory` dan `Android Studio directory` menempatkan adb dan emulator; biarkan kosong untuk deteksi otomatis. Begitu diatur, halaman menampilkan jalur terpecahkan dan asalnya (auto / env / override); `Reset to Auto-Detect` memulihkan bawaan. Untuk apa yang dibukanya, lihat Perangkat Android.

## Enkode aliran

- `Encoder` — Auto atau backend tertentu; server mendaftar encoder yang dapat dipakai dan mengaburkan yang tidak tersedia beserta alasannya;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — batas sisi penangkapan (4K / 2.5K / 2K);
- `Bitrate` — `Auto` menskalakan- luas dengan resolusi (laju dasar hingga 1080p, ~4× di 4K, dibatasi 40 Mbps); nilai tetap berlaku langsung pada aliran berjalan, sedangkan encoder / fps / resolusi berlaku untuk aliran yang dimulai setelahnya;
- kaki kartu menampilkan encoder / fps / resolusi / laju bit yang sedang berlaku.

![Web UI — pengaturan enkode aliran](images/remote-desktop-server-stream.jpg)

Klik `Save` untuk menerapkan perubahan.
