Spracheingabe verwandelt Sprache in Terminaleingabe: Sprechen statt Tippen, mit optionaler LLM-Politur des Transkripts vor dem Senden. Die Erkennung läuft als lokaler ASR-Server mit Modellen auf dem Gerät — Audio und Text bleiben auf Ihrer Maschine. Diese Seite behandelt die Serverseite; das Desktop-Erlebnis lebt unter Client verwenden.

## Die Karte Local ASR Server

Öffnen Sie den Bereich `Voice` der Web UI und aktivieren Sie die Spracherkennung. Die Karte `Local ASR Server` verwaltet den beaufsichtigten vterm-voice-Prozess:

- `Load on Startup` — den ASR-Server zusammen mit dem Backend starten;
- `Bind Address` (`127.0.0.1` nur localhost / `0.0.0.0` alle Schnittstellen) und `Port` (Standard `8398`) — wo der ASR-Endpunkt lauscht;
- `Model Directory` + `Local Model` — aus dem eingebauten Katalog wählen: Qwen3-ASR, Fun-ASR und Parakeet-Familien, jeweils in full / 8-bit / 4-bit-Varianten. `Download` holt das Modell von Hugging Face oder ModelScope ins Verzeichnis, mit live Fortschrittsbalken; manuelle `git clone`-Befehle werden als Fallback gezeigt, und Modelle auf der Platte werden als verfügbar markiert;
- `Device` — das Inferenz-Backend (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), wie vom Server gemeldet;
- `Access Token` — gemeinsames Geheimnis für den Endpunkt; der Würfel-Button erzeugt eines;
- `Save` speichert die Karte (Änderungen bleiben bis dahin ein lokaler Entwurf), und `Start Server` / `Stop Server` steuern den Prozess. Das Statussymbol im Kartentitel spiegelt Erreichbarkeit und Modellzustand (geladen / lädt / Fehler).

![Der Bereich Voice — die Karte Local ASR Server mit Modell, Gerät und Token](images/voice-server-local-asr.jpg)

> **CUDA-Konkurrenz**: Belegt bereits ein anderes CUDA-Inferenzprogramm die GPU (z. B. llama.cpp mit einem lokalen Modell), kann es beide drastisch verlangsamen, wenn vterm-voice ebenfalls `CUDA` nutzt. Setzen Sie `Device` in dem Fall auf `CPU` — diese kompakten ASR-Modelle sind auf der CPU schnell genug.

## Provider und Testen

Die Provider-Dropdown-Liste ASR-Addons — das eingebaute Addon `vibe-voice` zeigt auf diesen beaufsichtigten Server, sodass seine Auswahl der Weg ist, wie das Panel Ihre lokalen Modelle nutzt.

Der Bereich `Test` probiert alles ohne Client: `Record` (Mikrofon → Wellenform → Transkription) und `Live` (Streaming-Erkennung), jeweils mit Zeitmessungen und Kopieren-Button.

![Der Test-Bereich — die Tabs Record und Live](images/voice-server-test.jpg)
