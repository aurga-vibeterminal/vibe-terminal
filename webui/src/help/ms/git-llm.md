Kecerdasan panel Git didorong LLM: mesej commit satu klik, pembacaan diff berbantu AI dan penyelesaian konflik oleh ejen hunk demi hunk. Semuanya bergantung pada penyedia LLM yang dikonfigurasikan dalam Web UI.

## Mengkonfigurasi penyedia

Buka bahagian `LLM` Web UI:

- Pilih `Provider` dan isikan kredensialnya (medan ditakrifkan oleh manifes penyedia); konfigurasinya global — pengelasan terminal, mesej commit Git, pembantu sembang dan ciri AI lain berkongsi penyedia yang sama;
- Penyedia ialah addon jenis LLM: yang lazim terbina dalam, dan penyedia tersuai boleh dimuat naik sebagai zip dalam bahagian `Addons` (manifes mengisytiharkan medan konfigurasi yang dijadikan Web UI borang).

![Bahagian LLM Web UI — pilih penyedia dan isikan kredensialnya](images/git-llm-provider-config.jpg)

## Ciri LLM dalam Git

- **Penjanaan mesej commit** — klik butang AI di sebelah kotak commit; LLM membaca diff ter-stage dan menulis mesej, ditaip ke kotak dengan animasi. Mesej kekal boleh disunting — commit hanya apabila anda berpuas hati;
- **Analisis diff** — butang percikan dalam pengepala paparan diff menganalisis diff semasa;
- **Ejen AI konflik** — mod AI penyunting gabung rebase menyelesaikan hunk satu demi satu dengan penjelasan dan keyakinan (lihat Menyelesaikan Konflik).

![Mesej commit dijana AI ditaip ke dalam kotak commit](images/git-llm-commit-message.jpg)

> LLM hanya membaca kandungan repositori untuk menghasilkan teks; commit dan gabung sentiasa berlaku apabila anda mengesahkannya.
