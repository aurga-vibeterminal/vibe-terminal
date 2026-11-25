Rebase interaktif mengubah `git rebase -i` menjadi alur visual: seret untuk menata ulang antrean commit, terapkan commit satu per satu, selesaikan konflik secara visual — tanpa baris perintah.

## Memulai

Buka panel cabang dan ketik di kotak `Filter branches...` untuk menemukan cabang target, lalu klik kanan → `Rebase Current Branch onto '{branch}'` untuk merebase cabang saat ini ke target. Panel beralih ke tampilan rebase yang berjalan melalui empat fase:

![Pencarian cabang di panel, lalu memulai rebase ke target](images/git-rebase-search.jpg)

## 1. Menyunting antrean commit

Commit di cabang Anda yang tidak dimiliki target membentuk antrean:

- **Seret** untuk menatanya ulang;
- tiap commit punya dropdown aksi: `pick` (simpan) / `squash` (lipat ke sebelumnya, gabung pesan) / `fixup` (lipat, buang pesan) / `edit` (berhenti untuk amend) / `drop` (buang);
- commit yang di-squash dan di-drop melipat secara visual.

![Antrean commit rebase — tata ulang dengan seret, aksi tiap commit](images/git-rebase-queue.jpg)

## 2. Menerapkan

`Start Rebase` memutar ulang antrean ke target commit demi commit dengan bilah kemajuan langsung; tanpa konflik ia berjalan lurus sampai selesai. `Abort` mundur kapan saja.

## 3. Konflik (bila ada)

Konflik saat menerapkan beralih ke tampilan konflik — daftar berkas plus editor merge dengan empat modenya (`Merge` / `Split` / `Raw` / `AI`), tempat tiap hunk berkonflik mendapat keputusannya sendiri (`Ours` / `Theirs` / `Both` …; lihat Menyelesaikan Konflik). Setelah tiap berkas selesai, `Continue` melanjutkan rebase.

![Tampilan konflik — keputusan merge per hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Selesai

Saat selesai panel mengonfirmasi `Branch '{branch}' was rebased onto '{target}'` dan menampilkan kartu push — rebase menulis ulang riwayat, jadi cabang jarak jauh harus diperbarui paksa:

- `Force Push with Lease` — disarankan: menimpa hanya bila tak ada orang lain yang push, dan tak pernah memakai `--force` polos;
- `Not now` — lewati push; rebase lokal sudah selesai.

![Halaman selesai — Force Push with Lease atau lewati push](images/git-rebase-done.jpg)

> Sesi tetap dapat dipakai selama rebase; menutup panel tidak mengganggu yang sedang berjalan.
