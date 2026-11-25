Terjemahan menukar teks yang anda jumpai dalam klien kepada bahasa pilihan anda: balasan sembang AI, mesej commit, komen kod, atau apa-apa pemilihan dalam editor kod atau diff. Kerjanya dilakukan oleh model terjemahan yang dikonfigurasi di pelayan — klien hanya menghantar teks, memaparkan hasilnya dan menyimpannya dalam cache supaya anda boleh bertukar-tukar antara teks asal dan terjemahan.

## Di mana terjemahan tersedia

- **Panel Copilot** — setiap pengepala mesej (mesej anda, balasan model dan blok `Berfikir...`) mempunyai ikon terjemahan. Satu klik menggantikan kandungan mesej dengan terjemahannya; satu klik lagi memulihkan yang asal;
- **Draf mesej commit** — kotak commit dalam panel Git memaparkan butang terjemahan sebaik draf anda merentasi beberapa baris. Ia menterjemah keseluruhan draf di tempat, dan satu klik lagi mengembalikan kata-kata asal anda;
- **Popup butiran commit** — menghalaikan kursor ke atas commit dalam sejarah Git membuka popup butiran; ikon terjemahan di sebelah hash menukar mesej commit kepada bahasa anda;
- **Editor kod & pandangan diff** — pilih apa sahaja (pemilihan blok turut dikira), klik kanan dan pilih `Terjemah`. Lencana glob kecil menandakan baris yang diterjemahkan di gutter, dan kad dengan terjemahan terbuka di sebelah pemilihan.

## Cara ia berfungsi

1. Klien menghantar teks ke pelayan — bersama nama fail apabila teks dipilih daripada editor atau diff, supaya model tahu sintaks komen mana yang digunakan fail itu;
2. LLM terjemahan pelayan menterjemahkannya kepada bahasa sasaran yang dikonfigurasi dalam Web UI. Pemformatan Markdown (tajuk, senarai, jadual, blok kod) dikekalkan; dalam pemilihan fail, penanda komen seperti `//` atau `#` dikenali sebagai hiasan dan dibuang, manakala kod dan pengenal kekal tidak disentuh;
3. Klien memaparkan hasilnya sebagai Markdown di tempat permintaan datangnya — menggantikan terus untuk mesej sembang, atau dalam kad boleh tatal untuk pemilihan editor dan diff.

## Apa yang anda perlukan

- Pelayan dengan ciri terjemahan diaktifkan — konfigurasi sekali sahaja diterangkan dalam *Persediaan Pelayan* (bahagian `LLM` dalam Web UI);
- Klien desktop yang bersambung ke pelayan itu. Tiada apa yang dikonfigurasi di pihak klien: semua permukaan menggunakan tetapan pelayan yang sama secara automatik.

Jika terjemahan tidak dikonfigurasi, klien memaparkan ralat daripada pelayan (contohnya dalam kad terjemahan) dan bukannya hasil.
