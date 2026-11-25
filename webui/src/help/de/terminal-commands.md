Quick Commands heften die Build-/Test-/Deploy-Befehle eines Projekts ins Rechtsklick-Menü des Terminals — ein Klick zum Ausführen, überall verfügbar. Befehle werden pro Repository in `.vterm/commands.json` im Repository-Stamm verwaltet.

## commands.json erstellen

Rechtsklick im Terminal → `Commands`:

- Hat das Repository der Sitzung noch keine commands.json, bietet das Menü einen einzelnen Eintrag, `Generate commands.json` — ein Klick lässt den Server den Projekttyp erkennen (npm / Cargo / .NET / Xcode / Gradle-Android / generisch) und eine konservative Startvorlage schreiben. Eine bestehende nicht-leere Datei wird **niemals überschrieben**;
- Dateispeicherort: `.vterm/commands.json` im Git-Repository-Stamm (für nicht-Git-Ordner das aktuelle Arbeitsverzeichnis).

## Verfeinern

Die generierte Vorlage enthält nur die üblichen Befehle des erkannten Projekttyps. Lassen Sie Ihren Code-Agenten (zum Beispiel einen über das MCP dieses Servers verbundenen Editor-Agenten) das Projekt lesen und die Datei verfeinern:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

Auch ein Top-Level-Array wird akzeptiert; `env` ist optional und setzt Variablen vor dem Ausführen; der alte Schlüssel `value` wird weiterhin gelesen. Das optionale Feld `os` beschränkt einen Eintrag auf das Betriebssystem, auf dem `vterm-server` läuft: verwenden Sie `"win"` für Windows, `"linux"` für Linux oder `"osx"` für macOS. Fehlt `os`, ist es auf `"all"` gesetzt oder enthält einen unbekannten Wert, ist der Eintrag auf jeder Plattform verfügbar. Der Server beobachtet CWD und Änderungen an commands.json und überträgt sie — öffnen Sie das Menü erneut und die neuen Befehle sind da.

## Ausführen

Rechtsklick → `Commands` → einen Eintrag anklicken, um ihn in diesem Terminal auszuführen (optionales env wird vorher angewendet). Das Menü ist nicht verfügbar, während eine Agent-App in der Sitzung läuft, damit Befehlstext nicht in der UI der App landet.

## Beispiel: Android Build & Installation über WLAN

Eine komplette Anleitung an einem Gradle-Android-Projekt: das Wireless-ADB-Setup und den Build-and-Install-Fluss ins Rechtsklick-Menü heften, dann die App mit einem Klick neu bauen.

**1. Das Telefon auf Wireless-ADB umstellen.** Diese beiden Befehle sind einmal pro Gerät einzutippen — `adb tcpip 5555`, solange das Telefon am USB hängt, dann `adb connect` mit der Adresse des Telefons:

![Terminal — ein Android-Telefon mit adb tcpip 5555 und adb connect auf Wireless-ADB umstellen](images/terminal-commands-adb-setup.gif)

**2. Den Code-Agenten die Befehle schreiben lassen.** Öffnen Sie den Code-Agenten des Projekts (hier Claude Code) in einem Terminal und bitten Sie ihn, Build- & Installationsbefehle hinzuzufügen — er liest das Projektlayout und schreibt die Einträge in `.vterm/commands.json`:

![Terminal — Claude Code bearbeitet commands.json mit den Build- & Install-APK-Befehlen](images/terminal-commands-agent-edit.mp4)

**3. Aus dem Menü bauen und installieren.** Teilen Sie das Terminal, damit die Build-Ausgabe einen eigenen Bereich bekommt, dann Rechtsklick → `Commands` → der Eintrag — Gradle baut das APK und installiert es über WLAN auf dem Telefon:

![Terminal — Bereich teilen und den Android-Build-&-Install-Befehl aus dem Rechtsklick-Menü ausführen](images/terminal-commands-split-run.mp4)
