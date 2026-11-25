Menu klik kanan senarai cabang memusatkan push, checkout dan operasi cabang harian.

## Push

Klik kanan cabang → `Push '{branch}'...` membuka dialog push:

![Klik kanan cabang — entri Push](images/git-push-contextmenu.jpg)

- Ia menunjukkan pemetaan sasaran `local → remote`; kedua-dua nama boleh diubah (push ke cabang jauh sedia ada dengan nama lain meminta pengesahan);
- Pilihan: `Push all branches`, `Set upstream` (ikat upstream pada push pertama), `Push tags`, `Dry run`;
- `Force` penulis-sejarah dan `Force with lease` yang lebih selamat (menolak apabila orang lain sudah push — digemari). Operasi menulis semula sejarah, seperti push selepas rebase, memerlukan salah satunya;
- Petunjuk risiko tentang push paksa ditunjukkan dalam dialog.

![Dialog push — sasaran local → remote dan kotak semak pilihan](images/git-push-dialog.jpg)

## Checkout

Klik kanan cabang → `Checkout` untuk menukar cabang — cabang tempatan di-checkout terus, manakala cabang jauh atau tag membuka dialog checkout:

![Klik kanan cabang — entri Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — cabang tempatan yang dicipta, diisi awal daripada ref;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, cabang jauh sahaja), `Force` (`-f`), `Merge` (`--merge`).

![Dialog checkout — nama cabang dan pilihan](images/git-checkout-dialog.jpg)

Direktori kerja sesi mengikut peralihan; terminal dan panel Git mencerminkan cabang baharu serta-merta.

## Detached HEAD

Checkout commit tulen (cth. checkout commit sambil melayari sejarah) menanggalkan HEAD. Commit seterusnya kemudian membuka dialog `Detached HEAD`: pilih cabang sedia ada untuk commit, `+ Create new branch…` di tempat, atau `Skip` buat masa ini.

## Operasi lain

- **Stash** — stash semua perubahan ke timbunan (mesej pilihan) daripada pengepala perubahan; senarai `STASHES` menyokong `Pop` (terap dan buang) dan `Drop` (buang, tidak boleh diundur);
- **Memadam cabang/worktree** — melalui menu klik kanan entri, kedua-duanya dengan pengesahan tindakan tidak boleh diundur;
- **Sejarah & pemulihan** — `HISTORY` menyenaraikan commit; klik kanan untuk sejarah fail, diff terhadap pokok kerja, atau pulih ke semakan;
- **Salin** — nama cabang dan cincang commit disalin satu klik, berguna untuk menampal kepada ejen atau rakan sekerja.
