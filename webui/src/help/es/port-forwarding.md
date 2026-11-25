La redirección de puertos publica un servicio HTTP que se ejecuta en la máquina del servidor — un servidor de desarrollo, el panel de un dispositivo, cualquier cosa que escuche en una dirección simple `http://localhost:port` — en su propio puerto TLS, de modo que cualquier dispositivo que pueda alcanzar el servidor lo abra en un navegador como `https://<server-address>:<port>/`.

## Cómo funciona

- Una redirección mapea un **puerto de destino** — el servicio HTTP en esta máquina — a un **puerto redirigido** asignado del rango de puertos configurado (por defecto `62000`–`62999`);
- El puerto redirigido habla TLS: todo el tráfico se cifra con el **certificado autofirmado** del servidor — nada viaja en texto plano, ni siquiera por una red no confiable;
- Cualquier dispositivo que pueda alcanzar el servidor — en la LAN, o desde internet una vez redirigido el rango de puertos en el router — abre `https://<server-address>:<forwardedPort>/` y aterriza en el servicio local.

Las redirecciones se crean bajo demanda: el recurso compartido `Host with HTTP server` en un servidor remoto (conectado a través de un relay) solicita una automáticamente y te entrega la URL lista para abrir. No hace falta ninguna configuración manual.

## Gestionar redirecciones

![Interfaz web — la sección Port Forwarding: configuración del rango de puertos y la lista de redirecciones activas](images/port-forwarding-webui.jpg)

1. Abre la sección `Port Forwarding` de la interfaz web del servidor.
2. `Port Range Configuration` delimita el grupo del que se asignan los puertos redirigidos; los cambios se guardan automáticamente. Redúcelo cuando tu firewall o router solo redirige unos pocos puertos.
3. La lista en vivo muestra cada redirección como `:forwardedPort → :targetPort` con su número de conexiones activas.
4. El botón de parada de una redirección desmonta el listener de inmediato y desconecta sus conexiones activas; el siguiente recurso compartido asigna un puerto nuevo. `Refresh` vuelve a leer la lista.

## La advertencia del certificado

Las redirecciones se cifran con el certificado autofirmado propio del servidor — el mismo que protege la interfaz web y las conexiones de los clientes. Los navegadores no pueden verificarlo contra una autoridad de certificación, así que la primera visita a una URL redirigida muestra una advertencia de privacidad (*La conexión no es privada*). Es lo esperado y se puede continuar sin riesgo: el cifrado en sí no está comprometido, solo el origen del certificado no puede verificarse automáticamente. Para silenciar la advertencia de forma permanente, importa el certificado del servidor en el almacén de confianza de tu dispositivo.

![Navegador — la advertencia de certificado autofirmado no confiable; continúa mediante Advanced → Proceed para abrir la página](images/port-forwarding-cert-warning.jpg)

## Chrome en macOS muestra `ERR_ADDRESS_UNREACHABLE`

En macOS 15 (Sequoia) y posteriores, las aplicaciones de terceros necesitan permiso explícito para alcanzar dispositivos de la red local — cualquier dirección `192.168.x.x`, `10.x.x.x` o similar. Safari está exento por ser una aplicación del sistema, así que una URL redirigida puede abrirse sin problema en Safari mientras Chrome reporta `ERR_ADDRESS_UNREACHABLE` antes incluso de que comience el TLS. No es un problema del certificado ni un problema de la redirección en sí.

Para conceder acceso a Chrome:

1. Abre `Ajustes del Sistema` → `Privacidad y seguridad` → `Red local`.
2. Busca `Google Chrome` en la lista y actívalo.
3. Cierra Chrome por completo (`Cmd+Q`, no solo cerrar las ventanas) y vuelve a abrirlo.

Si el interruptor ya está activado, apágalo y enciéndelo de nuevo — puede quedarse atascado tras una actualización de Chrome o de macOS. Si el error persiste, comprueba si hay una extensión de VPN o proxy en Chrome que no enrute los rangos de direcciones privadas.

## Control de acceso

Lo que protege a un puerto redirigido depende de lo que haya detrás:

- **Los recursos integrados exigen token.** Los enlaces para compartir que genera el cliente — un recurso `Host with HTTP server` (`?share_token=…`) y la página de Settings (`?ticket=…`) — incluyen un token de acceso. Una conexión sin token válido y sin sesión válida se corta antes de servir nada. En la primera visita, el navegador lo canjea por una cookie de sesión segura específica de ese servicio y el token desaparece de la URL: no queda en la barra de direcciones ni en el historial.

- **Los tokens son de un solo uso y viven poco.** Un ticket de Settings funciona una sola vez y caduca al minuto; cuando un recurso compartido se detiene o se reinicia, los enlaces y sesiones anteriores dejan de funcionar. Trata el enlace completo como una credencial — quien lo tenga podrá acceder al servicio — y compártelo solo con personas de confianza.

- **Tus propios servicios conservan su propia autenticación.** Una redirección a cualquier otro servicio local cifra el tráfico con el certificado autofirmado del servidor pero no añade autenticación alguna: todo lo que el servicio sirva — páginas y API por igual — queda al alcance de cualquiera que pueda alcanzar el puerto. Pon autenticación a los servicios sensibles antes de exponerlos.
