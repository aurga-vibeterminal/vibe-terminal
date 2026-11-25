Di samping skrin desktop, desktop jauh boleh menstrim peranti Android — telefon sebenar melalui adb dan emulator AVD. Klien menunjukkannya dalam tetingkap berbingkai telefon dengan input sentuh, putaran dan kawalan kitaran hayat emulator jauh.

## Prasyarat

Pelayan perlu menemui rantaian alat Android:

- **SDK Android** (menyediakan `adb`) — diperlukan untuk peranti dan emulator;
- **Android Studio** — diperlukan untuk membut emulator AVD.

Kedua-duanya dikesan secara automatik secara lalai. Jika pengesanan gagal atau alat berada di luar lokasi standard, tetapkan `Android SDK directory` / `Android Studio directory` di bawah `Remote Desktop` → `Screen streaming tools`; perubahan berkuat kuasa serta-merta (diterapkan panas, tanpa but semula pelayan).

## Sumber peranti

Dengan laluan siap, pelayan mengenumerasi sumber Android tersedia secara automatik:

- **Peranti sebenar bersambung** — sambung melalui USB dengan nyahpepijat USB diaktifkan; apa sahaja yang kelihatan kepada `adb devices` boleh distrim;
- **Emulator AVD** — emulator yang berjalan muncul sendiri; anda turut boleh membut AVD terpilih atas permintaan daripada klien (tanpa perlu memulakannya secara manual dahulu) dan mematikannya dari jauh.

Setiap sumber mendapat entrinya sendiri dalam bar SCREENS klien.

## Orientasi

Butang putar tetingkap emulator menetapkan `user_rotation` sistem melalui `adb` — arah paparan peranti itu sendiri berubah, dan tetingkap muat semula ke resolusi baharu, menyesuaikan potret dan landskap sama rata.

## Pihak klien

Buka tab `Debug` dalam panel kiri klien desktop dan klik butang `Android Simulator` dalam bar SCREENS (hanya muncul apabila pelayan mendedahkan sumber Android) untuk membuka tetingkap emulator; pemilih peranti menukar rupa bingkai telefon dan `Rotate` membalik orientasi. Lihat Menggunakan Klien.

> Suntikan sentuh pada peranti sebenar menggunakan saluran input adb (Android 4.4+); sesetengah ROM tersuai pengeluar menyekatnya, dalam kes itu peranti hanya boleh dilihat.
