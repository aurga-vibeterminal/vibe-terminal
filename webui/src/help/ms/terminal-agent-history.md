Apabila anda menjalankan ejen kod (Claude Code, Codex, Kimi, Pi, OpenCode…) dalam terminal, perbualannya terperangkap dalam tetingkap terminal itu — tersasar antara output perintah, hilang begitu skrin bergerak. `Sejarah Perbualan Ejen Kod` mengangkat keseluruhan perbualan ke panel khusus: setiap pertukaran dipaparkan berstruktur, perbualan lampau boleh ditukar, mesej boleh diterjemah — dan anda boleh menghantar perintah ke terminal terus daripada panel.

## Membuka panel

Klik butang sembang (`AI`) di sebelah kanan bar tajuk untuk menogol panel Copilot. Menu lungsur di bahagiannya atas menyenaraikan **semua perbualan lampau** ejen yang berjalan dalam sesi terminal aktif, terbaharu dahulu; memilih satu memuatkannya.

![Panel Copilot — menu lungsur perbualan dan aliran mesej](images/terminal-agent-history.jpg)

## Cara ia berfungsi

- Pelayan tahu aplikasi ejen mana berjalan dalam terminal dan menghurai **fail sesi natifnya** (format JSONL/sejarah setiap ejen di bawah direktori projeknya); perubahan fail mengalir secara langsung, mengekalkan paparan perbualan terkini;
- Ejen disokong: `claude` / `codex` / `kimi` / `pi` / `opencode`; aplikasi lain belum mempunyai panel perbualan;
- Mesej membawa peranan (anda / ejen) dan jenis (teks / penaakulan **Thinking**), berserta nama model dan masa, dirender sebagai Markdown.

## Ciri panel

- **Tukar perbualan**: menu lungsur `Conversations` di atas bergerak antara sesi lampau, setiap entri menunjukkan ringkasan dan masanya;
- **Terjemah**: sebarang mesej ejen boleh diterjemah atas permintaan; terjemahan menggantikan teks di tempatnya (dan boleh ditogol kembali);
- **Hantar perintah terus**: taip dalam kotak di bahagian bawah (Shift+Enter untuk baris baharu) dan tekan Enter — perintah terus masuk ke terminal semasa, seperti menaip dalam kotak input ejen sendiri tanpa menduduki paparan terminal.

## Keserasian dengan ciri lain

- Panel dan terminal ialah dua paparan sesi yang sama: soalan ejen dalam terminal masih dijawab di sana (atau melalui Pengesahan Automatik); panel untuk menyemak sejarah, bukan untuk memintas interaksi;
- Digabung dengan `Terminal MCP` (halaman sebelumnya), ejen luaran turut boleh membaca skrin terminal dan menjalankan perintah — perbualan yang dipaparkan di sini berserta tindakan MCP membentuk rekod lengkap kerja ejen.
