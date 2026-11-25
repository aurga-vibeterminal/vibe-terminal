Penganalisis terminal pihak pelayan memerhati output terminal: ia mengesan gesaan perintah, mengenali penyiapan tugas, dan dengan LLM mengelasifikasikan apa yang dipaparkan terminal. `Auto Confirm` ialah ciri paling digunakannya — apabila terminal berhenti pada gesaan pengesahan biasa (seperti `Proceed? [Y/n]`), Enter dihantar untuk anda, jadi tugas panjang tidak lagi tersekat pada soalan.

## Konfigurasi

Bahagian `Terminal Analysis` Web UI:

- `Terminal observer` — suis utama analisis pihak pelayan;
- `Auto confirm` — menghantar Enter hanya untuk gesaan yang dikelaskan sebagai **pengesahan biasa**; soalan lain tidak pernah dijawab secara buta;
- `Analysis delay (seconds)` — berapa lama menunggu selepas ejen kosong sebelum menganalisis, mengelakkan panggilan LLM berlebihan antara tugas;
- `Confirmation grace (seconds)` — tetingkap masa ketika anda boleh mengambil alih selepas pengesahan dikesan;
- `Notify on completion` — memancarkan peristiwa apabila output dikelaskan sebagai tugas selesai (suapkan ke pemberitahuan).

> Nota halaman: apabila mod automatik ejen anda sendiri aktif, lebih baik matikan `Auto Confirm` pada klien supaya kedua-dua automasi tidak saling menjawab.

## Kebergantungan LLM

Pengelasan dilakukan oleh LLM: pilih `Provider` dan isikan kredensialnya di bahagian `LLM` (dikongsi oleh pengelasan terminal, mesej commit Git, pembantu sembang dan ciri AI lain). Tanpa LLM penganalisis masih berfungsi dengan ketepatan berkurangan — kekalkan auto confirm mati dalam kes itu.

![Halaman tetapan Terminal Analysis](images/terminal-auto-confirm.jpg)

![Halaman tetapan LLM — pilih penyedia dan isikan kredensial](images/terminal-auto-confirm-llm.jpg)
