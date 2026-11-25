Klien desktop membuka layar jarak jauh dari bilah SCREENS di atas panel Debug. Tiap layar hidup di jendela mengambangnya sendiri yang dapat dipakai berdampingan dengan terminal.

## Membuka bilah SCREENS

1. Sambungkan ke server target (langsung atau lewat relay);
2. klik tab `Debug` di bilah aktivitas kiri (ikon perkakas);
3. bilah `SCREENS` muncul di atas panel: satu tombol tampilan per layar desktop fisik di server, disusul sakelar `iOS Simulator` dan `Android Simulator`.

![Panel Debug — bilah SCREENS dengan tombol tampilan dan sakelar simulator](images/remote-desktop-client-screens.jpg)

## Jendela layar desktop

Klik tombol tampilan untuk membuka jendela layar itu:

- jendela bebas diubah ukurannya (hingga 800×600) dan gambar menata ulang bersamanya;
- server multi-monitor mendapat satu jendela per tampilan; beberapa dapat terbuka sekaligus;
- saat menyambung, jendela menampilkan penanda "menyiapkan desktop"; bila server kekurangan izin penangkapan, jendela langsung menampilkan instruksi izin platform;
- klik lagi tombol bilah atau tutup jendela untuk berhenti menonton (status aktif tombol tetap sinkron dengan jendela).

## Jendela simulator iOS

Klik `iOS Simulator` untuk membuka jendela berbingkai ponsel ukuran tetap (264×640 potret):

- pemilih perangkat di atas mengganti wujud ponsel; jendela menyesuaikan resolusi terpilih;
- `Rotate` membalik potret/lanskap (jendela 524×380 lanskap);
- memerlukan server macOS yang menjalankan simulator iOS Xcode.

## Jendela emulator Android

`Android Simulator` bekerja dengan cara sama: bingkai ponsel tetap, pemilih perangkat, tombol putar. Sumber yang dapat dialirkan mencakup emulator AVD yang berjalan dan perangkat nyata tersambung; AVD juga dapat di-boot atau dimatikan sesuai permintaan dari klien (lihat Perangkat Android).

> Jendela yang mengalirkan server localhost otomatis dikecualikan dari penangkapan layar agar tak terjadi cermin umpan balik; jendela yang mengalirkan mesin jarak jauh tetap dapat ditangkap agar rekaman luar tetap memperlihatkannya.
