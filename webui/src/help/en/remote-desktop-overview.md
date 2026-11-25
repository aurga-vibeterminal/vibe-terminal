Remote Desktop lets you watch and control the server's screens from the desktop client in real time: the machine's desktop is streamed into a window on your side, with your mouse and keyboard injected — as if you were sitting in front of it. Beyond desktop screens, Android emulators and iOS simulators can be viewed and driven inside phone-framed windows.

![Desktop client — a remote desktop window used alongside terminals](images/remote-desktop-overview-hero.webm)

## What it can do

- **Desktop screens** — each physical display on the server streams into its own resizable client window;
- **Android devices / emulators** — connected devices and AVD emulators appear as stream sources, viewable and touchable in a phone-framed window, with rotation and remote emulator boot/shutdown;
- **iOS simulator** (macOS servers) — view the Xcode iOS simulator in a phone-framed window with device picker and rotation.

## How it works

The server captures the screen and encodes it with configurable encoder, frame rate, resolution ceiling, and bitrate; the client decodes into floating windows and sends input events back over the existing encrypted client-server connection — no extra ports.

Platform preparation differs: Windows can run the screen server as a system service and needs a virtual display driver on headless machines; Linux uses a Portal consent plus session unlock; macOS requires Screen Recording permission. See Server Setup; Android devices are covered in Android Devices, and the client windows in Using the Client.
