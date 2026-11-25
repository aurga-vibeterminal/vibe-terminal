Saat rebase atau merge menabrak konflik, panel beralih ke tampilan konflik: daftar berkas konflik di kiri, editor merge di kanan. Berkas berkonflik juga muncul di daftar `CHANGES` dengan lencana `!` — mengekliknya melompat langsung ke tampilan ini alih-alih diff biasa. Editor menawarkan empat mode penyelesaian dan Anda dapat berpindah di antaranya kapan saja.

![Tampilan konflik — daftar berkas di kiri, editor merge di kanan](images/git-conflicts-view.jpg)

## Daftar berkas konflik

Berkas dikelompokkan menurut status: yang otomatis ter-merge sukses (otomatis ter-stage) dan yang masih butuh perhatian manual. Klik berkas untuk membukanya di editor; daftar diperbarui langsung seiring Anda menyelesaikan, sampai kosong.

## Empat mode penyelesaian

- **Merge** (bawaan) — konflik disajikan hunk demi hunk; per hunk pilih `take ours` / `take theirs` atau sunting hasil merge secara inline;
- **Split** — diff dipecah menjadi segmen baris yang lebih halus, sehingga kedua sisi dapat diadopsi sepotong-sepotong dalam satu hunk;
- **Raw** — penyuntingan polos dengan penanda konflik `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) untuk penyetelan tangan;
- **AI** — agen menyelesaikan berkas hunk demi hunk: LLM menganalisis tiap hunk konflik dan menghasilkan kartu solusi dengan penjelasan dan keyakinan. Anda dapat mengetik instruksi untuk mengarahkan strategi (mis. "utamakan theirs, pertahankan log"), jeda, setel di mode lain, dan lanjut — hunk yang sudah selesai dilewati.

Di tiap mode penyelesaian ditulis ke berkas di sesi Anda dan di-stage, persis seperti suntingan manual.

## Melanjutkan rebase

Begitu tiap konflik selesai, rebase berlanjut ke commit berikutnya; Anda juga dapat membatalkan kapan pun untuk kembali ke kondisi sebelum rebase.
