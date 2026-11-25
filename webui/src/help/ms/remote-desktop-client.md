Klien desktop membuka skrin jauh daripada bar SCREENS di bahagian atas panel Debug. Setiap skrin hidup dalam tetingkap terapung tersendiri yang boleh digunakan bersama terminal.

## Membuka bar SCREENS

1. Sambung ke pelayan sasaran (terus atau melalui relay);
2. klik tab `Debug` dalam bar aktiviti kiri (ikon alat);
3. bar `SCREENS` muncul di bahagian atas panel: satu butang paparan setiap skrin desktop fizik pada pelayan, diikuti togol `iOS Simulator` dan `Android Simulator`.

![Panel Debug — bar SCREENS dengan butang paparan dan togol simulator](images/remote-desktop-client-screens.jpg)

## Tetingkap skrin desktop

Klik butang paparan untuk membuka tetingkap skrin itu:

- tetingkap boleh disaiz semula secara bebas (hingga 800×600) dan gambar menata semula bersamanya;
- pelayan pelbagai monitor mendapat satu tetingkap setiap paparan; beberapa boleh terbuka serentak;
- semasa menyambung, tetingkap menunjukkan pemegang tempat "menyiapkan desktop"; jika pelayan tiada kebenaran tangkapan, tetingkap menunjukkan arahan kebenaran platform secara terus;
- klik butang bar semula atau tutup tetingkap untuk berhenti menonton (keadaan aktif butang kekal segerak dengan tetingkap).

## Tetingkap simulator iOS

Klik `iOS Simulator` untuk membuka tetingkap berbingkai telefon bersaiz tetap (264×640 potret):

- pemilih peranti di atas menukar rupa telefon; tetingkap muat semula ke resolusi terpilih;
- `Rotate` membalik potret/landskap (tetingkap 524×380 landskap);
- memerlukan pelayan macOS yang menjalankan simulator iOS Xcode.

## Tetingkap emulator Android

`Android Simulator` berfungsi dengan cara yang sama: bingkai telefon tetap, pemilih peranti, butang putar. Sumber boleh strim merangkumi emulator AVD yang berjalan dan peranti sebenar bersambung; AVD turut boleh dibut atau dimatikan atas permintaan daripada klien (lihat Peranti Android).

> Tetingkap yang menstrim pelayan localhost dikecualikan secara automatik daripada tangkapan skrin untuk mengelakkan cermin suap balik; tetingkap yang menstrim mesin jauh kekal boleh tangkap supaya rakaman luar masih menunjukkannya.
