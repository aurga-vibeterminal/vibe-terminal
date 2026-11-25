El escritorio remoto te permite observar y controlar las pantallas del servidor desde el cliente de escritorio en tiempo real: el escritorio de la máquina se transmite a una ventana de tu lado, con tu ratón y teclado inyectados — como si estuvieras sentado delante. Más allá de las pantallas de escritorio, los emuladores Android y simuladores iOS pueden verse y manejarse dentro de ventanas con marco de teléfono.

![Cliente de escritorio — una ventana de escritorio remoto usada junto a terminales](images/remote-desktop-overview-hero.webm)

## Qué puede hacer

- **Pantallas de escritorio** — cada pantalla física del servidor se transmite a su propia ventana de cliente redimensionable;
- **Dispositivos / emuladores Android** — los dispositivos conectados y emuladores AVD aparecen como fuentes, visibles y tocables en una ventana con marco de teléfono, con rotación y arranque/parada remota del emulador;
- **Simulador iOS** (servidores macOS) — ver el simulador iOS de Xcode en una ventana con marco de teléfono con selector de dispositivo y rotación.

## Cómo funciona

El servidor captura la pantalla y la codifica con codificador, tasa de fotogramas, techo de resolución y tasa de bits configurables; el cliente decodifica en ventanas flotantes y envía los eventos de entrada de vuelta por la conexión cifrada cliente-servidor existente — sin puertos extra.

La preparación por plataforma difiere: Windows puede ejecutar el servidor de pantalla como servicio del sistema y necesita un controlador de pantalla virtual en máquinas headless; Linux usa un consentimiento Portal más desbloqueo de sesión; macOS requiere permiso de grabación de pantalla. Véase Configuración del servidor; los dispositivos Android se cubren en Dispositivos Android, y las ventanas del cliente en Uso del cliente.
