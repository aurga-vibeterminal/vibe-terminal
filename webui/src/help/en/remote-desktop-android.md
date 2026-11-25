Besides desktop screens, remote desktop can stream Android devices — real phones over adb and AVD emulators. The client shows them in a phone-framed window with touch input, rotation, and remote emulator lifecycle control.

## Prerequisites

The server needs to find the Android toolchain:

- **Android SDK** (provides `adb`) — required for devices and emulators alike;
- **Android Studio** — required to boot AVD emulators.

Both are auto-detected by default. If detection fails or the tools live outside the standard locations, set `Android SDK directory` / `Android Studio directory` under `Remote Desktop` → `Screen streaming tools`; changes take effect immediately (hot-applied, no server restart).

## Device sources

With the paths in place, the server enumerates available Android sources automatically:

- **Connected real devices** — plug in over USB with USB debugging enabled; anything visible to `adb devices` can stream;
- **AVD emulators** — running emulators appear on their own; you can also boot a chosen AVD on demand from the client (no need to start it by hand first) and shut it down remotely.

Each source gets its own entry in the client's SCREENS bar.

## Orientation

The emulator window's rotate button sets the system `user_rotation` through `adb` — the device's own display direction changes, and the window re-fits to the new resolution, adapting to portrait and landscape alike.

## Client side

Open the `Debug` tab in the desktop client's left panel and click the `Android Simulator` button in the SCREENS bar (it only appears when the server exposes Android sources) to open the emulator window; the device picker switches the phone frame's appearance and `Rotate` flips orientation. See Using the Client.

> Touch injection on real devices uses adb's input channel (Android 4.4+); some vendor-customized ROMs restrict it, in which case the device is view-only.
