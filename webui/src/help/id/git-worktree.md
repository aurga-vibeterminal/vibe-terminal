Worktree men-checkout beberapa cabang satu repositori ke direktori terpisah: kembangkan paralel tanpa menyentuh pohon kerja utama, lalu merge cabang kembali dan bersih-bersih. Vibe Terminal mengubah seluruh siklus itu jadi tunjuk-dan-klik.

## Membuat worktree

1. Di daftar cabang panel Git, klik kanan cabang target (lokal atau jarak jauh) → `New Worktree`;
2. dialog menamai worktree baru dari cabang — sunting bila suka; konfirmasi, dan server membuat direktori worktree serta men-checkout cabang di sana;
3. **tab terminal baru terbuka otomatis** dengan direktori kerjanya diatur ke worktree baru — mulai mengembangkan secara terisolasi segera selagi pohon kerja utama tetap di cabangnya.

![Dialog New Worktree — nama bawaannya cabang](images/git-worktree-create.jpg)

![Tab baru yang dibuka untuk worktree yang baru dibuat](images/git-worktree-terminal.jpg)

## Bekerja paralel

Tiap worktree adalah direktori kerja yang sepenuhnya independen: build dan uji untuk cabang utama dan cabang fitur berjalan berdampingan di tab berbeda tanpa saling ganggu. Cabang yang di-checkout di worktree membawa penanda worktree yang menampilkan jalurnya di daftar cabang, dan commit serta push bekerja langsung dari tab worktree seperti pada cabang mana pun.

Klik kanan cabang worktree menambah dua entri khusus worktree: `Reveal in File Explorer` (sesi lokal) membuka direktorinya di pengelola berkas sistem, dan `Open in Terminal` membuka tab lain di sana.

## Merge kembali dan bersih-bersih

1. Commit dan push di dalam tab worktree seperti biasa;
2. alihkan tab utama ke cabang target (mis. `main`), klik kanan cabang fitur → `Merge into Current Branch` untuk membawa pekerjaannya;

   ![Menggabungkan cabang worktree kembali ke cabang utama](images/git-worktree-merge.jpg)

3. setelah ter-merge, klik kanan cabang → `Delete Worktree`: ini menghapus direktori worktree dan cabangnya bersamaan. Pastikan dulu semuanya sudah ter-merge atau ter-push — dialog memperingatkan lagi dan aksinya tak dapat dibatalkan.

   ![Konfirmasi Delete Worktree — direktori dan cabang dihapus bersamaan](images/git-worktree-delete.jpg)

