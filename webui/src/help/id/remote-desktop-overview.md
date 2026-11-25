Desktop jarak jauh memungkinkan Anda menyaksikan dan mengendalikan layar server dari klien desktop secara waktu nyata: desktop mesin dialirkan ke jendela di pihak Anda, dengan tetikus dan papan tik Anda disuntikkan — seolah Anda duduk di depannya. Di luar layar desktop, emulator Android dan simulator iOS dapat dilihat dan dikemudikan dalam jendela berbingkai ponsel.

![Klien desktop — jendela desktop jarak jauh dipakai berdampingan terminal](images/remote-desktop-overview-hero.webm)

## Apa yang bisa dilakukan

- **Layar desktop** — tiap tampilan fisik di server dialirkan ke jendela klien yang dapat diubah ukurannya sendiri;
- **Perangkat / emulator Android** — perangkat tersambung dan emulator AVD muncul sebagai sumber aliran, dapat dilihat dan disentuh di jendela berbingkai ponsel, dengan rotasi serta boot/matikan emulator dari jauh;
- **Simulator iOS** (server macOS) — melihat simulator iOS Xcode dalam jendela berbingkai ponsel dengan pemilih perangkat dan rotasi.

## Cara kerjanya

Server menangkap layar dan mengenkodenya dengan encoder, laju bingkai, batas resolusi, dan laju bit yang dapat diatur; klien mendekode ke jendela mengambang dan mengirim peristiwa masukan kembali lewat koneksi klien-server terenkripsi yang ada — tanpa port tambahan.

Persiapan per platform berbeda: Windows dapat menjalankan server layar sebagai layanan sistem dan membutuhkan penggerak tampilan virtual di mesin headless; Linux memakai persetujuan Portal plus buka kunci sesi; macOS memerlukan izin Perekaman Layar. Lihat Penyiapan Server; perangkat Android dibahas di Perangkat Android, dan jendela klien di Menggunakan Klien.
