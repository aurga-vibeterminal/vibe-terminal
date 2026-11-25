Rebase interaktif mengubah `git rebase -i` kepada aliran visual: seret untuk menyusun semula barisan commit, terapkan commit satu demi satu, selesaikan konflik secara visual — tanpa baris perintah.

## Memulakan

Buka anak tetingkap cabang dan taip dalam kotak `Filter branches...` untuk mencari cabang sasaran, kemudian klik kanan → `Rebase Current Branch onto '{branch}'` untuk merebase cabang semasa ke sasaran. Panel bertukar ke paparan rebase yang bergerak melalui empat fasa:

![Carian cabang dalam anak tetingkap, kemudian memulakan rebase ke sasaran](images/git-rebase-search.jpg)

## 1. Menyunting barisan commit

Commit pada cabang anda yang tiada pada sasaran membentuk barisan:

- **Seret** untuk menyusunnya semula;
- setiap commit mempunyai menu lungsur tindakan: `pick` (kekal) / `squash` (lipat ke sebelumnya, gabung mesej) / `fixup` (lipat, buang mesej) / `edit` (berhenti untuk meminda) / `drop` (buang);
- commit yang di-squash dan di-drop lipat secara visual.

![Barisan commit rebase — susun semula dengan seret, tindakan setiap commit](images/git-rebase-queue.jpg)

## 2. Menerapkan

`Start Rebase` memainkan semula barisan ke sasaran commit demi commit dengan bar kemajuan langsung; tanpa konflik ia berjalan terus hingga selesai. `Abort` berundur pada bila-bila masa.

## 3. Konflik (jika ada)

Konflik semasa penerapan bertukar ke paparan konflik — senarai fail berserta penyunting gabung dengan empat modnya (`Merge` / `Split` / `Raw` / `AI`), tempat setiap hunk berkonflik mendapat keputusannya sendiri (`Ours` / `Theirs` / `Both` …; lihat Menyelesaikan Konflik). Setelah setiap fail selesai, `Continue` menyambung semula rebase.

![Paparan konflik — keputusan gabung setiap hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Selesai

Semasa penyiapan panel mengesahkan `Branch '{branch}' was rebased onto '{target}'` dan menunjukkan kad push — rebase menulis semula sejarah, jadi cabang jauh mesti dikemas kini secara paksa:

- `Force Push with Lease` — disyorkan: menulis ganti hanya apabila tiada orang lain yang push dan tidak pernah menggunakan `--force` tulen;
- `Not now` — langkau push; rebase tempatan sudah lengkap.

![Halaman selesai — Force Push with Lease atau langkau push](images/git-rebase-done.jpg)

> Sesi kekal boleh digunakan semasa rebase; menutup panel tidak mengganggu yang sedang berjalan.
