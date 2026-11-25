Semuanya berada dalam bahagian `Remote Desktop` Web UI; halaman menunjukkan tetapan yang terpakai pada platform pelayan.

## Suis utama

`Enable Remote Desktop` — sekali hidup, klien jauh yang bersambung boleh melihat desktop mesin ini dan menghantar input.

## Tetapan platform

**Windows**

- `Windows Service` — mendaftarkan pelayan skrin sebagai perkhidmatan sistem mula-auto supaya desktop jauh bertahan merentasi but semula tanpa sesi daftar masuk. `Install` / `Uninstall` mencetuskan gesaan UAC pada pelayan (butang menunjukkan kemajuan sementara menunggu); lencana status membezakan berjalan / terpasang / tidak terpasang.
- `Mesin headless (tiada monitor)` — tanpa paparan tersambung (atau sedar), Windows tidak mengekalkan desktop boleh tangkap. Halaman memautkan muat turun pemacu sumber terbuka Virtual Display Driver (ARM64 / x64 dipilih automatik); pasangkannya supaya desktop maya sentiasa wujud.

![Web UI — bahagian Remote Desktop di Windows (kad perkhidmatan dan headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — membuka kunci sesi terkunci secara automatik apabila strim bermula (di mana persekitaran menyokongnya);
- tangkapan skrin melalui Portal; `Reset Permission` pada kad `Screen Recording Permission` mengosongkan persetujuan Portal tersimpan supaya anda boleh memilih skrin lain.
- `Auto-start on Login` (dalam bahagian `Pelayan`) — pada hos dengan pengurus paparan, pelayan bermula bersama sesi grafik, jadi tangkapan mewarisi persekitaran desktop penuh; pada hos headless ia bermula semasa but sebelum sebarang daftar masuk. Untuk mencapai desktop apabila tiada sesiapa yang mendaftar masuk secara tempatan, dayakan juga daftar masuk automatik dalam pengurus paparan (GDM, SDDM, …); demi keselamatan, kunci skrin automatik boleh kekal hidup — sesi dan pelayan terus berjalan semasa terkunci, dan `Unlock on Connect` (atau `loginctl unlock-session`) membuka kunci sesi dari jauh.
![Web UI — kad Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — sistem memerlukan kelulusan eksplisit untuk tangkapan. Klik `Grant Permission` untuk membuka dialog sistem; `Reset Permission` pulih daripada keadaan tersekat.

## Alat penstriman skrin (sokongan Android)

`Android SDK directory` dan `Android Studio directory` menempatkan adb dan emulator; biarkan kosong untuk pengesanan automatik. Sekali ditetapkan, halaman menunjukkan laluan terlerai dan asalnya (auto / env / override); `Reset to Auto-Detect` memulihkan lalai. Untuk apa yang dibukanya, lihat Peranti Android.

## Pengekodan strim

- `Encoder` — Auto atau bahagian belakang tertentu; pelayan menyenaraikan pengekod boleh guna dan mengelabukan yang tidak tersedia beserta sebabnya;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — silau pihak tangkapan (4K / 2.5K / 2K);
- `Bitrate` — `Auto` menskala mengikut keluasan dengan resolusi (kadar asas hingga 1080p, ~4× pada 4K, had 40 Mbps); nilai tetap terpakai serta-merta kepada strim berjalan, manakala pengekod / fps / resolusi terpakai kepada strim yang dimulakan selepasnya;
- kaki kad menunjukkan pengekod / fps / resolusi / kadar bit yang sedang berkuat kuasa.

![Web UI — tetapan pengekodan strim](images/remote-desktop-server-stream.jpg)

Klik `Save` untuk menerapkan perubahan.
