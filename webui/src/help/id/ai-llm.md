Bagian `LLM` Web UI menugaskan penyedia LLM untuk tiap fitur AI — klasifikasi terminal, pesan commit Git, penyelesaian konflik, polesan teks suara, dan terjemahan. Penyedia itu sendiri adalah addon bertipe LLM yang dikelola di bagian `Addons`.

## Penyedia adalah addon

Penyedia LLM hidup di grup `LLM` bagian `Addons`:

- Penyedia **OpenAI Compatible LLM** terpasang bawaan; penyedia kustom dapat diunggah sebagai zip (tombol `Upload`). Manifest addon mendeklarasikan bidang konfigurasi (kunci API, base URL, model, …) yang diubah Web UI menjadi formulir;
- penyedia hanya muncul di pemilih fitur selama addonya **diaktifkan** — nyalakan dulu di bagian Addons;
- tombol pensil pada kartu addon menyunting sumbernya; addon bawaan yang Anda ubah dapat digulung balik dengan `Reset`.

![Bagian Addons — penyedia LLM adalah addon yang dapat diaktifkan, disunting, atau diunggah](images/ai-llm-addons.jpg)

## Konfigurasi per fitur

Tiap kartu fitur di bagian `LLM` punya sakelar aktifkan dan pilihan penyedianya sendiri, sehingga pekerjaan berbeda dapat memakai model berbeda:

- **Terminal Classification** — mengklasifikasikan status terminal saat pola bawaan tak cukup. `Max Consecutive Failures` menonaktifkan LLM untuk sesi setelah sekian kegagalan beruntun (minimum 3);
- **Git Commit Messages** — menghasilkan pesan commit dari perubahan ter-stage;
- **Git Conflict Resolve** — menyelesaikan konflik rebase/merge hunk demi hunk;
- **Voice Text Polish** — memoles transkrip input suara sebelum dikirim;
- **Translation** — menerjemahkan teks ke bahasa sasaran terpilih.

Buka kartu untuk memilih `Provider`-nya dan mengisi bidang penyedia. Konfigurasi tersimpan bahkan saat fitur nonaktif dan baru berlaku begitu diaktifkan. Ketika beberapa fitur memakai addon penyedia yang sama, tombol salin/tempel di samping `Provider Configuration` menggandakan nilai bidang penyedia itu di antara fitur-fitur tersebut (lihat di bawah).

Fitur Terminal dan Git menawarkan bahasa keluaran dengan opsi `Auto` (model memutuskan); terjemahan memakai daftar sasaran English/简体中文 eksplisit.

![Bagian LLM — tiap fitur memilih penyedia dan kredensialnya sendiri](images/ai-llm-features.jpg)

## Salin/tempel antar fitur

Setiap fitur menyimpan salinan sendiri dari bidang penyedia, jadi memakai satu penyedia untuk beberapa fitur biasanya berarti mengetik ulang base URL, kunci API, dan model yang sama di tiap kartu. Judul `Provider Configuration` punya dua tombol kecil yang menghilangkan pengulangan:

- **Salin** — menyalin nilai bidang yang saat ini ditampilkan di kartu ini, beserta addon penyedia tempatnya berada;
- **Tempel** — hanya muncul di kartu bila penyedia yang dipilih di sana adalah addon yang sama dengan konfigurasi yang disalin; klik untuk mengisi bidang kartu dengan nilai yang disalin.

Untuk berbagi satu endpoint antara, misal, pesan commit Git dan terjemahan: pilih penyedianya dan isi bidangnya di kartu pertama, tekan **Salin**, buka kartu lain (dengan penyedia yang sama dipilih) lalu tekan **Tempel**. Nilai yang ditempel disimpan di konfigurasi milik fitur itu sendiri seperti pengeditan bidang lainnya dan disimpan bersamanya. Papan klip hanya berlaku dalam halaman — ia dikosongkan saat Anda meninggalkan bagian LLM.

![Tombol Salin dan Tempel di judul Provider Configuration — menggandakan bidang satu penyedia antar fitur yang memakai addon sama](images/ai-llm-duplicate-config.jpg)

## Penyedia kompatibel OpenAI

Penyedia bawaan bekerja dengan API obrolan kompatibel OpenAI mana pun. Bidang kunci:

- `Base URL` / `API Key` / `Model` — endpoint, kredensial, nama model;
- `Extra Body` — objek JSON yang digabungkan ke tiap badan permintaan, untuk parameter spesifik model;
- `Max Input Tokens` / `Max Output Tokens` — anggaran prompt dan batas penyelesaian;
- `Timeout` — batas waktu per permintaan dalam detik; naikkan untuk model lokal yang lambat.

## Model yang disarankan

Pekerjaan AI ini adalah prompt kecil yang peka latensi — model cepat yang "pas" mengalahkan model penalaran besar:

- **Lokal: Qwen3.5 9B** — titik manis untuk pembuatan pesan commit Git, klasifikasi terminal, terjemahan, dan polesan suara. Layani dengan endpoint kompatibel OpenAI (Ollama, vLLM, llama.cpp) dan arahkan `Base URL` ke sana;
- **API: DeepSeek V4 Flash (`deepseek-v4-flash`)** — cukup cepat untuk pemakaian interaktif dan mampu menangani semua fitur di atas.

> Mode berpikir menambah detik latensi dan jarang membantu tugas pendek ini — matikan di `Extra Body` untuk pengalaman lebih baik, mis. `{"enable_thinking": false}` untuk Qwen atau `{"thinking": {"type": "disabled"}}` untuk DeepSeek (kunci persisnya bergantung tumpukan penyajian Anda; JSON digabungkan apa adanya ke badan permintaan).
