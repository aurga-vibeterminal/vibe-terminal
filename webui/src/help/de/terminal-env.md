Mit Umgebungsvariablen pflegen Sie gruppierte Variablensätze pro Projekt (`ANDROID_HOME`, API-Schlüssel, Proxy-Einstellungen) und wenden sie mit einem Klick auf jede Terminal-Sitzung an oder löschen sie. Sie leben in der Serverkonfiguration; Werte werden im Ruhezustand verschlüsselt.

## Pflege in der Web UI

Der Bereich `Environment Variables` organisiert sie als Baum:

- **Gruppen** — reine Container, schachtelbar (Rechtsklick → `New Group`), zur Organisation nach Projekt oder Szenario;
- **Sammlungen** — die Blätter des Baums und die tatsächlichen Variablenhalter (`New Leaf`); nur Sammlungen lassen sich auf ein Terminal anwenden;
- Die Schlüssel/Wert-Paare jeder Sammlung werden über `Edit Variables` bearbeitet (`Add Variable`); Werte sind standardmäßig verborgen;
- Sammlungen lassen sich duplizieren, um Varianten abzuleiten (derselbe Satz mit einem geänderten Schlüssel).

![Die Seite Environment Variables — der Gruppen-/Sammlungsbaum](images/terminal-env-tree.jpg)

## Auf ein Terminal anwenden

Rechtsklick im Terminal → `Envs`:

- Das Untermenü spiegelt den Baum aus der Web UI; klappen Sie eine Sammlung auf und wählen Sie `Apply`, um ihre Variablen in die aktuelle Sitzung zu injizieren, oder `Clear`, um sie wieder zu entfernen;
- Neu gestartete Prozesse sehen die Variablen (bereits laufende Vordergrundprogramme nicht);
- Sammlungen erscheinen unter ihren Gruppen zusammengeklappt; der ganze `Envs`-Eintrag verschwindet, solange der Baum keine Sammlungen hat.

![Terminal-Rechtsklick — Anwenden und Löschen im Envs-Untermenü](images/terminal-env-menu.mp4)
