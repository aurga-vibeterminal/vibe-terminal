Spraakinvoer zet spraak om in terminalinvoer: praat in plaats van typen, met optionele LLM-bijschaving van de transcriptie voor verzending. Herkenning draait als een lokale ASR-server met modellen op het apparaat — audio en tekst blijven op jouw machine. Deze pagina behandelt de serverzijde; de desktopervaring staat in De client gebruiken.

## De kaart Local ASR Server

Open de sectie `Voice` van de Web UI en schakel spraakherkenning in. De kaart `Local ASR Server` beheert het begeleide vterm-voice-proces:

- `Load on Startup` — de ASR-server samen met de backend starten;
- `Bind Address` (`127.0.0.1` alleen localhost / `0.0.0.0` alle interfaces) en `Port` (standaard `8398`) — waar het ASR-endpoint luistert;
- `Model Directory` + `Local Model` — kies uit de ingebouwde catalogus: de families Qwen3-ASR, Fun-ASR en Parakeet, elk in full / 8-bit / 4-bit-varianten. `Download` haalt het model van Hugging Face of ModelScope naar de map met een live voortgangsbalk; handmatige `git clone`-opdrachten worden als alternatief getoond, en modellen die al op schijf staan worden als beschikbaar gemarkeerd;
- `Device` — de inferentie-backend (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), zoals door de server gerapporteerd;
- `Access Token` — gedeeld geheim voor het endpoint; de dobbelsteenknop genereert er een;
- `Save` persisteert de kaart (bewerkingen blijven tot dan toe een lokaal concept), en `Start Server` / `Stop Server` besturen het proces. Het statuspictogram in de kaartkop weerspiegelt bereikbaarheid en modelstatus (geladen / laadt / fout).

![De sectie Voice — de kaart Local ASR Server met model, device en token](images/voice-server-local-asr.jpg)

> **CUDA-conflict**: wanneer een ander CUDA-inferentieprogramma de GPU al bezet (bijv. llama.cpp met een lokaal model), kan het ook voor vterm-voice inschakelen van `CUDA` beide drastisch vertragen. Zet in dat geval `Device` op `CPU` — deze compacte ASR-modellen zijn op CPU snel genoeg.

## Provider en testen

De provider-dropdown somt ASR-addons op — de ingebouwde addon `vibe-voice` wijst naar deze begeleide server, dus die kiezen is hoe het paneel jouw lokale modellen gebruikt.

Het gebied `Test` probeert alles zonder client: `Record` (microfoon → golfvorm → transcriptie) en `Live` (streamende herkenning), elk met tijden en een kopieerknop.

![Het Test-gebied — de tabbladen Record en Live](images/voice-server-test.jpg)
