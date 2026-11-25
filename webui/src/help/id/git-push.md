Menu klik kanan daftar cabang memusatkan push, checkout, dan operasi cabang sehari-hari.

## Push

Klik kanan cabang → `Push '{branch}'...` membuka dialog push:

![Klik kanan cabang — entri Push](images/git-push-contextmenu.jpg)

- Menampilkan pemetaan target `local → remote`; salah satu nama dapat diubah (push ke cabang jarak jauh yang ada dengan nama lain meminta konfirmasi);
- Opsi: `Push all branches`, `Set upstream` (ikat upstream saat push pertama), `Push tags`, `Dry run`;
- `Force` penulis-riwayat dan `Force with lease` yang lebih aman (menolak saat orang lain sudah push — disarankan). Operasi penulis-riwayat, seperti push setelah rebase, memerlukan salah satunya;
- Petunjuk risiko tentang force push ditampilkan di dialog.

![Dialog push — target local → remote dan kotak centang opsi](images/git-push-dialog.jpg)

## Checkout

Klik kanan cabang → `Checkout` untuk berganti cabang — cabang lokal di-checkout langsung, sedangkan cabang jarak jauh atau tag membuka dialog checkout:

![Klik kanan cabang — entri Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — cabang lokal yang dibuat, terisi dari ref;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, hanya cabang jarak jauh), `Force` (`-f`), `Merge` (`--merge`).

![Dialog checkout — nama cabang dan opsi](images/git-checkout-dialog.jpg)

Direktori kerja sesi mengikuti peralihan; terminal dan panel Git langsung mencerminkan cabang baru.

## Detached HEAD

Checkout commit telanjang (mis. checkout commit sambil menelusuri riwayat) melepaskan HEAD. Commit berikutnya lalu membuka dialog `Detached HEAD`: pilih cabang yang ada untuk commit, `+ Create new branch…` di tempat, atau `Skip` dulu.

## Operasi lain

- **Stash** — stash semua perubahan ke tumpukan (pesan opsional) dari header perubahan; daftar `STASHES` mendukung `Pop` (terapkan dan hapus) dan `Drop` (buang, tak dapat dibatalkan);
- **Menghapus cabang/worktree** — lewat menu klik kanan entri, keduanya dengan konfirmasi aksi tak dapat dibatalkan;
- **Riwayat & pemulihan** — `HISTORY` mendaftar commit; klik kanan untuk riwayat berkas, diff terhadap pohon kerja, atau pulihkan ke revisi;
- **Salin** — nama cabang dan hash commit tersalin sekali klik, praktis untuk menempel ke agen atau rekan.
