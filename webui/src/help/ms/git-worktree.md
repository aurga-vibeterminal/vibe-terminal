Worktree men-checkout berbilang cabang satu repositori ke direktori berasingan: bangunkan secara selari tanpa menyentuh pokok kerja utama, kemudian gabungkan cabang semula dan bersihkan. Vibe Terminal mengubah keseluruhan kitaran itu kepada tunjuk-dan-klik.

## Mencipta worktree

1. Dalam senarai cabang panel Git, klik kanan cabang sasaran (tempatan atau jauh) → `New Worktree`;
2. dialog menamakan worktree baharu daripada cabang — sunting jika suka; sahkan, dan pelayan mencipta direktori worktree dan men-checkout cabang di sana;
3. **tab terminal baharu terbuka secara automatik** dengan direktori kerjanya ditetapkan ke worktree baharu — mula membangun secara terpencil serta-merta selagi pokok kerja utama kekal pada cabangnya.

![Dialog New Worktree — nama lalainya cabang](images/git-worktree-create.jpg)

![Tab baharu dibuka untuk worktree yang baharu dicipta](images/git-worktree-terminal.jpg)

## Bekerja secara selari

Setiap worktree ialah direktori kerja yang sepenuhnya bebas: binaan dan ujian untuk cabang utama dan cabang ciri berjalan sebelah menyebelah dalam tab berbeza tanpa gangguan. Cabang yang di-checkout dalam worktree membawa penanda worktree yang menunjukkan laluannya dalam senarai cabang, dan commit serta push berfungsi terus daripada tab worktree seperti pada sebarang cabang.

Klik kanan cabang worktree menambah dua entri khusus worktree: `Reveal in File Explorer` (sesi tempatan) membuka direktorinya dalam pengurus fail sistem, dan `Open in Terminal` membuka tab lain di sana.

## Menggabung semula dan membersihkan

1. Commit dan push dalam tab worktree seperti biasa;
2. tukar tab utama ke cabang sasaran (cth. `main`), klik kanan cabang ciri → `Merge into Current Branch` untuk membawa kerjanya;

   ![Menggabungkan cabang worktree semula ke cabang utama](images/git-worktree-merge.jpg)

3. setelah tergabung, klik kanan cabang → `Delete Worktree`: ini membuang direktori worktree dan cabangnya bersama. Pastikan dahulu semuanya tergabung atau ter-push — dialog memberi amaran semula dan tindakan tidak boleh diundur.

   ![Pengesahan Delete Worktree — direktori dan cabang dibuang bersama](images/git-worktree-delete.jpg)

