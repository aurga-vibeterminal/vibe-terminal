Bahagian `LLM` Web UI menugaskan penyedia LLM kepada setiap ciri AI — pengelasan terminal, mesej commit Git, penyelesaian konflik, polesan teks suara dan terjemahan. Penyedia itu sendiri ialah addon jenis LLM yang diurus dalam bahagian `Addons`.

## Penyedia ialah addon

Penyedia LLM hidup dalam kumpulan `LLM` bahagian `Addons`:

- Penyedia **OpenAI Compatible LLM** terbina dalam; penyedia tersuai boleh dimuat naik sebagai zip (butang `Upload`). Manifes addon mengisytiharkan medan konfigurasi (kunci API, URL asas, model, …) yang dijadikan Web UI borang;
- penyedia hanya muncul dalam pemilih ciri selagi addonya **diaktifkan** — hidupkannya dahulu dalam bahagian Addons;
- butang pensil pada kad addon menyunting sumbernya; addon terbina dalam yang anda ubah boleh digulung balik dengan `Reset`.

![Bahagian Addons — penyedia LLM ialah addon yang boleh diaktifkan, disunting atau dimuat naik](images/ai-llm-addons.jpg)

## Konfigurasi setiap ciri

Setiap kad ciri dalam bahagian `LLM` mempunyai togol aktifkan dan pilihan penyedianya sendiri, supaya kerja berbeza boleh menggunakan model berbeza:

- **Terminal Classification** — mengelasifikasikan keadaan terminal apabila corak terbina dalam tidak mencukupi. `Max Consecutive Failures` melumpuhkan LLM untuk sesi selepas banyak kegagalan berturutan itu (minimum 3);
- **Git Commit Messages** — menjana mesej commit daripada perubahan ter-stage;
- **Git Conflict Resolve** — menyelesaikan konflik rebase/gabung hunk demi hunk;
- **Voice Text Polish** — memoles transkrip input suara sebelum dihantar;
- **Translation** — menterjemahkan teks ke bahasa sasaran terpilih.

Buka kad untuk memilih `Provider`-nya dan mengisi medan penyedia. Konfigurasi disimpan walaupun ciri dilumpuhkan dan hanya berkuat kuasa sekali diaktifkan. Apabila beberapa ciri menggunakan addon penyedia yang sama, butang salin/tampal di sebelah `Provider Configuration` mengklon nilai medan penyedia itu antara ciri-ciri tersebut (lihat di bawah).

Ciri Terminal dan Git menawarkan bahasa output dengan pilihan `Auto` (model memutuskan); terjemahan menggunakan senarai sasaran English/简体中文 eksplisit.

![Bahagian LLM — setiap ciri memilih penyedia dan kredensialnya sendiri](images/ai-llm-features.jpg)

## Salin/tampal antara ciri

Setiap ciri menyimpan salinan sendiri medan penyedianya, jadi menggunakan satu penyedia untuk beberapa ciri biasanya bermakna menaip semula URL asas, kunci API dan model yang sama di setiap kad. Tajuk `Provider Configuration` mempunyai dua butang kecil yang memadamkan pengulangan:

- **Salin** — menyalin nilai medan yang dipaparkan dalam kad ini, bersama addon penyedia yang menjadi punca nilainya;
- **Tampal** — hanya muncul dalam kad apabila penyedia yang dipilih di sana ialah addon yang sama dengan konfigurasi disalin; klik untuk mengisi medan kad dengan nilai disalin.

Untuk berkongsi satu titik akhir antara, katakan, mesej commit Git dan terjemahan: pilih penyedianya dan isi medannya dalam kad pertama, tekan **Salin**, buka kad lain (dengan penyedia yang sama dipilih) dan tekan **Tampal**. Nilai ditampal disimpan dalam konfigurasi milik ciri itu sendiri seperti suntingan medan lain dan disimpan bersamanya. Papan keratan hanya dalam halaman — ia dikosongkan apabila anda meninggalkan bahagian LLM.

![Butang Salin dan Tampal dalam tajuk Provider Configuration — menggandakan medan satu penyedia antara ciri yang menggunakan addon sama](images/ai-llm-duplicate-config.jpg)

## Penyedia serasi OpenAI

Penyedia terbina dalam berfungsi dengan sebarang API sembang serasi OpenAI. Medan kunci:

- `Base URL` / `API Key` / `Model` — titik akhir, kredensial, nama model;
- `Extra Body` — objek JSON digabungkan ke setiap badan permintaan, untuk parameter khusus model;
- `Max Input Tokens` / `Max Output Tokens` — belanjakan gesaan dan silau penyiapan;
- `Timeout` — had masa setiap permintaan dalam saat; naikkan untuk model lokal yang perlahan.

## Model disyorkan

Kerja AI ini ialah gesaan kecil yang peka latensi — model pantas yang "tepat" mengalahkan model penaakulan besar:

- **Lokal: Qwen3.5 9B** — titik manis untuk penjanaan mesej commit Git, pengelasan terminal, terjemahan dan polesan suara. Hidangkan dengan titik akhir serasi OpenAI (Ollama, vLLM, llama.cpp) dan halakan `Base URL` kepadanya;
- **API: DeepSeek V4 Flash (`deepseek-v4-flash`)** — cukup pantas untuk penggunaan interaktif dan mampu mengendalikan semua ciri di atas.

> Mod berfikir menambah beberapa saat latensi dan jarang membantu tugas pendek ini — matikannya dalam `Extra Body` untuk pengalaman lebih baik, cth. `{"enable_thinking": false}` untuk Qwen atau `{"thinking": {"type": "disabled"}}` untuk DeepSeek (kunci tepat bergantung pada timbunan perkhidmatan anda; JSON digabungkan kata demi kata ke badan permintaan).
