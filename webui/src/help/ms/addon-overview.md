Tambahan memperluas pelayan dengan keupayaan baharu tanpa menyentuh teras: penyedia LLM, backend pengecaman suara, pengemas kini DNS dinamik, permukaan alat MCP tambahan dan pengesanan aplikasi ejen kod semuanya hidup sebagai tambahan. Bahagian `Addons` dalam Web UI menguruskannya.

## Kategori

Setiap tambahan mengisytiharkan jenis, dan bahagian mengumpulkan kad mengikutnya:

- **Applications** — mengajar pemerhati terminal mengenali CLI ejen kod (Claude Code, Codex, Kimi Code, Pi dan OpenCode terbina dalam): pengesanan proses, status sibuk/luang, cangkuk automasi;
- **LLM Providers** — backend penyiapan sembang yang dipilih oleh ciri AI (penyedia serasi OpenAI terbina dalam);
- **Voice / ASR** — backend pengecaman suara (tambahan terbina dalam `vibe-voice` menunjuk pelayan ASR setempat yang diawasi);
- **Dynamic DNS** — mengekalkan hostname mengikut alamat semasa (Cloudflare terbina dalam);
- **MCP** — permukaan alat MCP tambahan dilayan pada hos MCP (EasyEDA Pro terbina dalam);
- yang lain jatuh ke kumpulan **Other**.

## Tambahan terbina dalam dan dimuat naik

- Tambahan **terbina dalam** datang bersama program dan sentiasa tersedia. Manifest dan sumbernya kekal tulen — sumber yang ditimpa boleh dipulihkan pada bila-bila masa;
- Tambahan **dimuat naik** adalah milik anda: zip memuat manifest dan sumber JavaScript, dipasang melalui butang `Upload Addon`;
- Lencana `Modified` menandai tambahan yang sumbernya anda timpa.

Setiap kad memaparkan jenis, lencana dan togol pengaktifan; bila dikembangkan memperlihatkan keterangan, pengarang, medan konfigurasi dan kod sumber.

![Bahagian Addons — kad berkelompok dengan lencana dan togol pengaktifan](images/addon-section.jpg)

## Konfigurasi

Manifest tambahan mengisytiharkan medan konfigurasinya, dan kad yang dikembangkan menjadikannya borang. Nilai disimpan setiap tambahan dan hanya berkesan selagi ia diaktifkan; medan kata laluan kekal disamarkan. Menyahaktifkan tambahan mengekalkan konfigurasinya — bila diaktifkan semula, anda menyambung dari tempat anda berhenti.
