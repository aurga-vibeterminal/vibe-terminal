Penganalisis terminal sisi server mengawasi keluaran terminal: mendeteksi prompt perintah, mengenali penyelesaian tugas, dan dengan LLM mengklasifikasikan apa yang ditampilkan terminal. `Auto Confirm` adalah fiturnya yang paling banyak dipakai — ketika terminal berhenti di prompt konfirmasi biasa (seperti `Proceed? [Y/n]`), Enter dikirim untuk Anda, sehingga tugas panjang tidak lagi tersendat pada pertanyaan.

## Konfigurasi

Bagian `Terminal Analysis` di Web UI:

- `Terminal observer` — sakelar utama analisis sisi server;
- `Auto confirm` — mengirim Enter hanya untuk prompt yang diklasifikasikan sebagai **konfirmasi biasa**; pertanyaan lain tidak pernah dijawab buta;
- `Analysis delay (seconds)` — berapa lama menunggu setelah agen menganggur sebelum menganalisis, menghindari panggilan LLM berlebih di antara tugas;
- `Confirmation grace (seconds)` — jendela waktu ketika Anda dapat mengambil alih setelah konfirmasi terdeteksi;
- `Notify on completion` — memancarkan peristiwa saat keluaran diklasifikasikan sebagai tugas selesai (umpankan ke notifikasi).

> Catatan halaman: saat mode otomatis agen Anda sendiri aktif, lebih baik matikan `Auto Confirm` di klien agar dua otomatisasi tidak saling menjawab.

## Ketergantungan LLM

Klasifikasi dilakukan oleh LLM: pilih `Provider` dan isi kredensialnya di bagian `LLM` (dipakai bersama klasifikasi terminal, pesan commit Git, asisten obrolan, dan fitur AI lain). Tanpa LLM penganalisis tetap bekerja dengan akurasi berkurang — dalam kasus itu biarkan auto confirm mati.

![Halaman pengaturan Terminal Analysis](images/terminal-auto-confirm.jpg)

![Halaman pengaturan LLM — pilih penyedia dan isi kredensial](images/terminal-auto-confirm-llm.jpg)
