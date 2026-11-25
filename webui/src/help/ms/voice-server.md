Input suara menukar tutur kepada input terminal: bercakap selain menaip, dengan polesan LLM pilihan untuk transkrip sebelum dihantar. Pengecaman berjalan sebagai pelayan ASR lokal dengan model pada peranti — audio dan teks kekal pada mesin anda. Halaman ini meliputi pihak pelayan; pengalaman desktop ada di Menggunakan Klien.

## Kad Local ASR Server

Buka bahagian `Voice` Web UI dan aktifkan pengecaman suara. Kad `Local ASR Server` mengurus proses vterm-voice yang diawasi:

- `Load on Startup` — memulakan pelayan ASL bersama bahagian belakang;
- `Bind Address` (`127.0.0.1` localhost sahaja / `0.0.0.0` semua antara muka) dan `Port` (lalai `8398`) — tempat titik akhir ASR mendengar;
- `Model Directory` + `Local Model` — pilih daripada katalog terbina dalam: keluarga Qwen3-ASR, Fun-ASR dan Parakeet, masing-masing dalam varian full / 8-bit / 4-bit. `Download` mengambil model daripada Hugging Face atau ModelScope ke direktori dengan bar kemajuan langsung; perintah `git clone` manual ditunjukkan sebagai sandaran, dan model yang sudah pada cakera ditanda tersedia;
- `Device` — bahagian belakang inferens (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), seperti yang dilaporkan pelayan;
- `Access Token` — rahsia dikongsi untuk titik akhir; butang dadu menjana satu;
- `Save` mengekalkan kad (suntingan kekal draf lokal sehingga itu), dan `Start Server` / `Stop Server` mengawal prosesnya. Ikon status dalam tajuk kad mencerminkan kecapaian dan keadaan model (dimuatkan / memuatkan / ralat).

![Bahagian Voice — kad Local ASR Server dengan model, peranti dan token](images/voice-server-local-asr.jpg)

> **Perebutan CUDA**: apabila program inferens CUDA lain sudah menduduki GPU (cth. llama.cpp menjalankan model lokal), membiarkan vterm-voice turut menggunakan `CUDA` boleh memperlahankan kedua-duanya secara drastik. Dalam kes itu tetapkan `Device` kepada `CPU` — model ASR padat ini cukup pantas pada CPU.

## Penyedia dan pengujian

Menu lungsur penyedia menyenaraikan addon ASR — addon `vibe-voice` terbina dalam menunjuk kepada pelayan terawasi ini, jadi memilihnya ialah cara panel menggunakan model lokal anda.

Kawasan `Test` mencuba segalanya tanpa klien: `Record` (mikrofon → gelombang → transkripsi) dan `Live` (pengecaman strim), masing-masing dengan pemasaan dan butang salin.

![Kawasan Test — tab Record dan Live](images/voice-server-test.jpg)
