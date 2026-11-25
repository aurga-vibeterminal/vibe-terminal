Panel Git membangun kendali versi ke dalam klien: staging, commit, mengelola cabang, worktree, rebase interaktif, penyelesaian konflik, dan push — tanpa meninggalkan jendela terminal Anda. Semuanya beroperasi pada repositori Git sesi terminal saat ini: di direktori mana pun sesi berjalan, itulah repositori yang ditampilkan panel.

## Membukanya

Klik tab `Git` di bilah aktivitas kiri (data repositori hanya muncul saat sesi aktif berada dalam repositori Git). Repositori tab mengikuti sesi aktifnya; breadcrumb repositori memungkinkan melompat antar repositori induk dan anak, dan submodul didaftar terpisah.

![Panel Git — perubahan, riwayat, dan cabang](images/git-overview-panel.jpg)

## Apa yang bisa dilakukan

- **Staging & commit** — stage/unstage per berkas atau massal, buang perubahan, commit dengan pesan, pesan commit yang dihasilkan AI (lihat Bantuan LLM);
- **Riwayat** — riwayat commit dan per berkas, bandingkan dan pulihkan antar revisi;
- **Cabang** — checkout, buat, dan hapus cabang lokal/jarak jauh serta tag;
- **Worktree** — checkout cabang ke pohon kerja terisolasi (tab terminal baru terbuka otomatis), merge kembali saat selesai (lihat Worktree);
- **Rebase interaktif** — atur ulang/squash/sunting antrean commit secara visual dengan penyelesaian konflik (lihat Rebase Interaktif);
- **Penyelesaian konflik** — editor merge dengan empat mode, termasuk agen AI yang menyelesaikan hunk satu per satu (lihat Menyelesaikan Konflik);
- **Push & lainnya** — dialog push lengkap, stash, penanganan detached-HEAD (lihat Push, Checkout & Lainnya).

Halaman-halaman berikut mencakup tiap area dalam urutan itu.
