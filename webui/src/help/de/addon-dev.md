Es gibt zwei Ebenen der Anpassung: den Quelltext eines vorhandenen Add-ons patchen oder ein eigenes nach dem Vorbild eines eingebauten schreiben und hochladen.

## Quelltext-Hotpatch

Der Bleistift-Knopf auf jeder Add-on-Karte öffnet den JavaScript-Quelltext zum Bearbeiten. Speichern legt einen Snapshot an, der das Original überschreibt — bei eingebauten wie hochgeladenen Add-ons — und die Änderung greift sofort, ohne Neustart. Ideal, um einen Prompt zu schärfen, eine URL zu korrigieren oder eine Idee live auszuprobieren.

`Reset` verwirft den Snapshot und stellt den ursprünglichen Quelltext wieder her (nur bei eingebauten — ein hochgeladenes Add-on ist ohnehin Ihres).

![Add-on-Quelltext bearbeiten — der gespeicherte Snapshot überschreibt das Original](images/addon-edit-source.jpg)

## Ein eigenes Add-on schreiben

Ein Add-on besteht aus zwei Dateien:

- `<id>.json` — das Manifest: `id`, `name`, `description`, `author`, `email`, `date`, `type` sowie `config_fields` (key, label, Feldtyp, Standardwert, Beschreibung), aus denen die Web UI ein Formular macht;
- `<id>.js` — der Quelltext: eine gewöhnliche JavaScript-Datei (ausgeführt von der eingebetteten QuickJS-Engine, mit einer `addon.get_config()`-Brücke zu den konfigurierten Feldern), die die von ihrem Typ erwarteten Funktionen exportiert — z. B. `chat_completion()` für einen LLM-Provider, Werkzeuglisten und Bridge-Aufrufe für ein MCP-Add-on.

Der schnellste Einstieg ist der Export eines vorhandenen Add-ons: der `Export`-Knopf auf seiner Karte lädt ein Zip mit Manifest und Quelltext zusammen herunter. Wählen Sie das eingebaute Add-on, das Ihrem Ziel am nächsten kommt (sie liegen auch in `crates/vterm-server/src/addon/builtins/`), passen Sie id, name, type und die exportierten Funktionen an und installieren Sie das Ergebnis mit `Upload Addon`. Hochgeladene Add-ons lassen sich mit dem Papierkorb-Knopf wieder löschen.

![Add-on exportieren — Manifest und JavaScript-Quelltext als ein Zip](images/addon-export.jpg)

## Speicherort

Hochgeladene Add-ons und Quelltext-Snapshots liegen als `<id>.json` + `<id>.js` im Ordner `addons/` des Konfigurationsverzeichnisses und überstehen Neustarts. Bei einer eingebauten id wird nur die `.js`-Überschreibung beachtet — das eingebaute Add-on behält sein eigenes Manifest.
