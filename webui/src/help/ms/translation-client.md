Setelah pelayan dikonfigurasi, setiap permukaan klien menawarkan terjemahan melalui ikon glob yang sama. Hasil disimpan dalam cache mengikut item, jadi bertukar-tukar tidak pernah mengulangi permintaan.

## Mesej dalam panel sembang

Setiap pengepala mesej mempunyai ikon terjemahan — di sebelah masa pada mesej anda, di kanan nama penghantar dan masa pada balasan model, dan di kanan label `Berfikir...` pada blok penaakulan:

- **Klik pertama** menghantar Markdown mentah mesej ke pelayan dan menukar kandungan mesej dengan terjemahannya. Ikon menyala selagi terjemahan dipaparkan;
- **Klik lagi** memulihkan teks asal;
- Terjemahan kekal dengan mesej selagi perbualan masih terbuka, termasuk blok `Berfikir...` setelah dikembangkan.

Mesej terjemahan dipaparkan sebagai Markdown biasa, jadi senarai, jadual dan blok kod mengekalkan bentuknya.

![Mesej sembang dengan ikon terjemahan di pengepala dan kandungan terjemahan di bawahnya](images/translation-chat-message.jpg)

## Draf mesej commit

Dalam kotak commit panel Git, butang terjemahan muncul di sebelah butang AI sebaik draf anda melebihi satu baris:

- **Klik** untuk menterjemah keseluruhan draf — teks terjemahan menggantikan draf dalam kotak sebagai pratonton baca-sahaja;
- **Klik semula** untuk kembali kepada kata-kata asal anda (draf itu sendiri tidak pernah diubah);
- Menaip dilumpuhkan selagi pratonton dipaparkan; kembali dahulu, kemudian teruskan menyunting.

![Kotak commit dengan butang terjemahan dan pratonton terjemahan](images/translation-commit-draft.jpg)

## Popup butiran commit

Halaikan kursor ke atas commit dalam sejarah Git untuk membuka popup butiran. Di sebelah butang salin hash terdapat ikon terjemahan: klik untuk menterjemah mesej commit dalam popup; klik semula untuk melihat yang asal. Mesej terjemahan dipaparkan seperti asal — baris subjek, kandungan Markdown dan blok kod termasuk.

![Popup butiran commit — butang salin dan terjemah pada baris hash](images/translation-git-tip.jpg)

## Pemilihan dalam editor kod

Pilih teks dalam editor kod — pemilihan biasa dan blok (segi empat) sama-sama berfungsi — klik kanan dan pilih `Terjemah` (entri pertama menu):

1. Lencana glob pejal muncul di atas nombor baris pertama yang dipilih;
2. Kad terjemahan terus terbuka dengan pemegang tempat *Translating…* dan terisi apabila pelayan menjawab;
3. Kad itu melekat: kekal di tempatnya semasa anda menatal, dan hanya butang tutupnya, `Esc` atau klik di luar yang menutupnya. Menghalaikan kursor ke lencana membukanya semula.

Setiap pemilihan yang diterjemah mendapat lencananya sendiri, dan menterjemah baris yang sama semula semata-mata memaparkan semula kad yang dicache. Lencana hilang apabila anda membuka fail lain.

![Pemilihan dalam editor kod dengan Terjemah dalam menu konteks](images/translation-editor-menu.jpg)

## Pemilihan dalam pandangan diff

Pandangan diff berfungsi dengan cara yang sama: pilih baris dalam kandungan diff, klik kanan, `Terjemah`. Lencana menggantikan nombor baris baharu pada baris pertama yang dipilih, dan kad hasilnya berlabuh di bawahnya. Pemilihan dalam mod sunting turut boleh diterjemah; bertukar ke fail atau commit lain mengosongkan lencana.

![Diff dengan pemilihan yang diterjemah — lencana gutter dan kad terjemahan](images/translation-diff-card.jpg)

## Apa yang diterjemah

Pelayan memberitahu model dari mana teks itu datang:

- **Prosa** (mesej sembang, mesej commit) — pemformatan Markdown dikekalkan dan hanya bahasa semula jadi diterjemah;
- **Pemilihan fail** (editor, diff) — nama fail ikut bersama permintaan, jadi model tahu penanda komen fail itu (`//`, `#`, `--`, …). Penanda komen dibuang dan teks komen diterjemah sebagai prosa; Markdown dalam komen (tajuk, senarai, blok kod) kekal. Jika pemilihannya kod biasa, hanya komen dan literal rentetan diterjemah — pengenal dan struktur kekal tidak disentuh.
