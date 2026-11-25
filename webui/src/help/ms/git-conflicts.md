Apabila rebase atau gabung menemui konflik, panel bertukar ke paparan konflik: senarai fail konflik di kiri, penyunting gabung di kanan. Fail berkonflik turut muncul dalam senarai `CHANGES` dengan lencana `!` — mengkliknya melompat terus ke paparan ini dan bukannya diff tulen. Penyunting menawarkan empat mod penyelesaian dan anda boleh bertukar antaranya pada bila-bila masa.

![Paparan konflik — senarai fail di kiri, penyunting gabung di kanan](images/git-conflicts-view.jpg)

## Senarai fail konflik

Fail dikumpulkan mengikut keadaan: yang tergabung automatik dengan jayanya (ter-stage automatik) dan yang masih memerlukan perhatian manual. Klik fail untuk membukanya dalam penyunting; senarai dikemas kini secara langsung ketika anda menyelesaikan, sehingga kosong.

## Empat mod penyelesaian

- **Merge** (lalai) — konflik dibentangkan hunk demi hunk; setiap hunk pilih `take ours` / `take theirs` atau sunting hasil gabung sebaris;
- **Split** — diff dipecahkan kepada segmen aras baris yang lebih halus, supaya kedua-dua pihak boleh diambil sepotong demi sepotong dalam satu hunk;
- **Raw** — penyuntingan tulen dengan penanda konflik `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) untuk penalaan tangan;
- **AI** — ejen menyelesaikan fail hunk demi hunk: LLM menganalisis setiap hunk berkonflik dan menghasilkan kad penyelesaian dengan penjelasan dan keyakinan. Anda boleh menaip arahan untuk mengemudikan strategi (cth. "utamakan theirs, kekal pernyataan log"), jeda, tala dalam mod lain dan sambung — hunk yang sudah selesai dilangkau.

Dalam setiap mod penyelesaian ditulis ke fail dalam sesi anda dan di-stage, tepat seperti penyuntingan manual.

## Meneruskan rebase

Setelah setiap konflik selesai, rebase bergerak ke commit seterusnya; anda juga boleh menghenti pada bila-bila titik untuk kembali ke keadaan pra-rebase.
