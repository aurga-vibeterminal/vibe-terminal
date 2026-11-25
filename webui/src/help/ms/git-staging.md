Kawasan `CHANGES` panel Git meliputi kitaran commit harian: menyemak perubahan, staging, membuang, stash dan commit dengan mesej.

## Stage dan unstage

- Perubahan tidak ter-stage disenaraikan di bawah `CHANGES`, setiap fail ditanda lencana status — `M` diubah, `A` ditambah, `D` dipadam, `?` tidak dijejak, `!` konflik (baris submodule membawa tag `SUB` tambahan);
- Melayari kursor ke atas baris mendedahkan butang tindakan sebaris: `+` me-stage fail tidak ter-stage, `−` men-unstage yang ter-stage, dan `×` pada baris tidak ter-stage membuangnya; butang `Stage All Changes` / `Unstage All Changes` dalam pengepala bahagian mengendalikan semuanya sekaligus;
- `Ctrl`-klik menogol fail ke pilihan pelbagai dan `Shift`-klik memilih julat — butang tindakan kemudiannya terpakai pada setiap fail terpilih;
- Mengklik nama fail membuka diffnya supaya anda menyemak perubahan sebelum memutuskan;
- Klik kanan fail menawarkan `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` dan pintasan `cd` yang melompatkan terminal ke folder fail.

![Bahagian STAGED CHANGES — fail ter-stage dengan butang Unstage setiap baris](images/git-staging-staged.jpg)

## Commit

1. Stage apa yang patut masuk;
2. taip mesej dalam kotak commit (`Ctrl+Enter` commit) atau klik butang AI untuk menjana satu (lihat Bantuan LLM);
3. klik `Commit`.

![Kotak commit — input mesej dengan butang jana AI](images/git-staging-commit.jpg)

Selepas commit, layari kursor ke pengepala sejarah — bar dengan nama cabang di atas sejarah commit — dan butang `Fetch`, `Pull` dan `Push` pudar masuk di kanannya. `Push` menghantar cabang semasa ke upstreamnya serta-merta; untuk pilihan seperti `Force with lease`, gunakan dialog push senarai cabang (lihat Push, Checkout & Lagi).

![Melayari kursor ke pengepala sejarah mendedahkan butang Fetch, Pull dan Push](images/git-staging-push.jpg)

## Stash

- Butang arkib dalam pengepala `CHANGES` membuka dialog `Stash All Changes`, tempat mesej pilihan menggambarkan stash;
- klik kanan fail tidak ter-stage terpilih menawarkan `Stash This File` (atau `Stash <n> Selected Files` untuk pilihan pelbagai) untuk stash laluan itu sahaja — baris submodule dikecualikan;
- stash terkumpul di bawah `STASHES` sebagai entri `WIP on <cabang>`; membuka satu menyenaraikan fail yang ditangkap, dan mengklik fail menunjukkan diffnya;
- melayari kursor ke baris stash menunjukkan `Apply` (pulih dan kekalkan), `Pop` (terap dan buang) dan `Drop` (buang tanpa terap) — `Pop` dan `Drop` meminta pengesahan dahulu, dan menu klik kanan menambah `View Diff`;
- bahagian `STASHES` kekal tersembunyi selagi senarai stash kosong.

![Dialog Stash All Changes dengan mesej pilihannya](images/git-staging-stash-all.jpg)

![Bahagian STASHES — stash terbuka ke failnya, dengan butang Apply, Pop dan Drop](images/git-staging-stashes.jpg)

![Pop menerapkan stash dan membuangnya daripada senarai selepas pengesahan](images/git-staging-stash-pop.jpg)

## Membuang

`Discard Changes` pada fail tidak ter-stage membuang semua perubahan belum commit di dalamnya — ada dialog pengesahan dan tindakannya **tidak boleh diundur**.

> Commit hanya merangkumi kandungan ter-stage; perubahan tidak ter-stage kekal dalam pokok kerja.
