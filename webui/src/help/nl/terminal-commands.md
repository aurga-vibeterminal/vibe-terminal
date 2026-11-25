Snelle commando's pinnen de build/test/deploy-commando's van een project in het rechtsklikmenu van de terminal — één klik om uit te voeren, overal beschikbaar. Commando's worden per repository onderhouden in `.vterm/commands.json` in de repositorywortel.

## commands.json aanmaken

Rechtsklik in een terminal → `Commands`:

- Als de repository van de sessie nog geen commands.json heeft, biedt het menu één invoer aan, `Generate commands.json` — klikken laat de server het projecttype detecteren (npm / Cargo / .NET / Xcode / Gradle-Android / generiek) en een behoudend startsjabloon schrijven. Een bestaand niet-lege bestand wordt **nooit overschreven**;
- Bestandslocatie: `.vterm/commands.json` in de Git-repositorywortel (de huidige werkmap voor niet-Git-mappen).

## Verfijnen

Het gegenereerde sjabloon bevat alleen de gebruikelijke commando's van het gedetecteerde projecttype. Laat je code-agent (bijv. een editor-agent verbonden met de MCP van deze server) het project lezen en het bestand verfijnen:

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

Een array op het hoogste niveau wordt ook geaccepteerd; `env` is optioneel en stelt variabelen in voordat de opdracht draait; de oudere sleutel `value` wordt nog gelezen. Het optionele veld `os` beperkt een invoer tot het besturingssysteem waarop `vterm-server` draait: gebruik `"win"` voor Windows, `"linux"` voor Linux of `"osx"` voor macOS. Als `os` ontbreekt, op `"all"` staat of een onbekende waarde bevat, is de invoer op elk platform beschikbaar. De server volgt de CWD en wijzigingen aan commands.json en zendt ze uit — open het menu opnieuw en de nieuwe commando's staan er.

## Uitvoeren

Rechtsklik → `Commands` → klik op een invoer om die in die terminal uit te voeren (optionele env wordt eerst toegepast). Het menu is niet beschikbaar zolang een agent-app in de sessie draait, zodat opdrachttekst niet in de UI van de app belandt.

## Voorbeeld: Android build & installatie via Wi-Fi

Een volledige wandeling op een Gradle-Android-project: pin de draadloze-ADB-setup en de build-en-install-flow in het rechtsklikmenu, en bouw de app daarna met één klik opnieuw.

**1. Zet de telefoon op draadloze ADB.** Deze twee commando's worden één keer per apparaat getypt — `adb tcpip 5555` terwijl de telefoon op USB zit, daarna `adb connect` met het adres van de telefoon:

![Terminal — een Android-telefoon op draadloze ADB zetten met adb tcpip 5555 en adb connect](images/terminal-commands-adb-setup.gif)

**2. Laat de code-agent de commando's schrijven.** Open de code-agent van het project (hier Claude Code) in een terminal en vraag build- en installatiecommando's toe te voegen — hij leest de projectindeling en schrijft de invoeren in `.vterm/commands.json`:

![Terminal — Claude Code bewerkt commands.json met de build- & install-APK-commando's](images/terminal-commands-agent-edit.mp4)

**3. Bouwen en installeren vanuit het menu.** Splits de terminal zodat de builduitvoer een eigen deelvenster krijgt, dan rechtsklik → `Commands` → de invoer — Gradle assembleert de APK en installeert hem via Wi-Fi op de telefoon:

![Terminal — een deelvenster splitsen en het Android-build- & installatiecommando uit het rechtsklikmenu uitvoeren](images/terminal-commands-split-run.mp4)
