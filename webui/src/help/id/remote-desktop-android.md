Di samping layar desktop, desktop jarak jauh dapat mengalirkan perangkat Android — ponsel nyata lewat adb dan emulator AVD. Klien menampilkannya dalam jendela berbingkai ponsel dengan masukan sentuh, rotasi, dan kendali siklus hidup emulator dari jauh.

## Prasyarat

Server perlu menemukan rantai alat Android:

- **SDK Android** (menyediakan `adb`) — wajib untuk perangkat dan emulator;
- **Android Studio** — wajib untuk mem-boot emulator AVD.

Keduanya terdeteksi otomatis secara bawaan. Bila deteksi gagal atau alat berada di luar lokasi standar, atur `Android SDK directory` / `Android Studio directory` di `Remote Desktop` → `Screen streaming tools`; perubahan langsung berlaku (diterapkan panas, tanpa restart server).

## Sumber perangkat

Dengan jalur siap, server menghitung sumber Android yang tersedia secara otomatis:

- **Perangkat nyata tersambung** — sambungkan lewat USB dengan debugging USB aktif; apa pun yang terlihat `adb devices` dapat dialirkan;
- **Emulator AVD** — emulator yang berjalan muncul sendiri; Anda juga dapat mem-boot AVD pilihan sesuai permintaan dari klien (tanpa perlu menyalakannya manual dulu) dan mematikannya dari jauh.

Tiap sumber mendapat entrinya sendiri di bilah SCREENS klien.

## Orientasi

Tombol putar jendela emulator mengatur `user_rotation` sistem lewat `adb` — arah tampilan perangkat itu sendiri berubah, dan jendela menyesuaikan diri ke resolusi baru, beradaptasi ke potret maupun lanskap.

## Sisi klien

Buka tab `Debug` di panel kiri klien desktop dan klik tombol `Android Simulator` di bilah SCREENS (hanya muncul saat server mengekspos sumber Android) untuk membuka jendela emulator; pemilih perangkat mengubah wujud bingkai ponsel dan `Rotate` membalik orientasi. Lihat Menggunakan Klien.

> Penyuntikan sentuh pada perangkat nyata memakai kanal masukan adb (Android 4.4+); sebagian ROM kustom pabrikan membatasinya, dalam kasus itu perangkat hanya untuk dilihat.
