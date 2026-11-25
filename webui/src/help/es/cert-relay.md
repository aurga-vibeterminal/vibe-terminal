Por defecto, un servidor compartido cifra su conexión aguas arriba con el relay pero no verifica quién está al otro lado — la misma postura de «confianza en el primer uso» que aceptar un certificado autofirmado en un navegador. El **TLS estricto del relé** cambia eso: el servidor compartido comprueba el certificado del relay contra una CA raíz que usted proporciona y se niega a conectarse a menos que la cadena, la validez y el nombre de host pasen todas las comprobaciones. Esto hace que los ataques de intermediario sobre el enlace del relay sean prácticamente imposibles, al precio de un paso manual: entregar al servidor compartido la CA raíz del relay. Aplica la misma idea que la página anterior, Verificación estricta en clientes, al enlace aguas arriba del servidor compartido.

## Dónde se configura

La verificación estricta se configura en el servidor compartido — la máquina cuya `Relay Address` apunta al relay. En su interfaz web, abra la sección `Server`: debajo de la `Relay Secret Key` hay una fila `TLS estricto del relé` que muestra el estado `On` / `Off` actual. Despliéguela para ver:

1. `CA raíz del relé ascendente` — un campo multilínea para el certificado público de la CA raíz del relay;
2. Un conmutador `Off` / `On` — `On` solo se puede elegir cuando el PEM se interpreta como un certificado válido.

![Interfaz web — el bloque TLS estricto del relé desplegado en la sección Server: campo CA raíz del relé ascendente con el conmutador Off/On y el botón Apply](images/cert-relay-strict-tls.jpg)

Los ajustes del relay nunca se guardan a mitad de edición: revise los valores y pulse `Apply`. Guardar reconecta de inmediato el relay aguas arriba con los nuevos ajustes.

## Llevar la CA raíz al servidor compartido

Dos formas, desde la misma fuente (la interfaz web del propio relay):

- **Pegar una cadena de uso compartido** — pulse el `QR code` en la interfaz web del servidor relay. La cadena incorpora la CA raíz del relay junto con la dirección y la clave, así que al pegarla en el campo `Relay Address` del servidor compartido también se autorrellena el campo de la CA (vea la Guía de configuración en Relay). El modo estricto sigue desactivado hasta que usted revise la CA y lo active;
- **Copiar y pegar** — pulse `Copiar CA raíz` en la página `Certificados y TLS` del relay y pegue el PEM en el campo `CA raíz del relé ascendente`.

La CA es la misma para todos los servidores compartidos y nunca cambia entre renovaciones del certificado — basta con configurarla una vez por relay.

Antes de confiar en la CA, confirme que tiene la auténtica: la página `Certificados y TLS` del relay muestra la huella SHA-256 de la CA raíz. Compárela con el PEM que está a punto de pegar. Si difieren, no active la verificación estricta — consiga la CA por un canal de confianza en su lugar.

## Qué comprueba el modo estricto

Con el conmutador en `On`, la conexión `RelayPrimary` aguas arriba debe presentar un certificado que:

1. Encadene con la CA raíz configurada — un certificado firmado por cualquier otra cosa, incluidas las raíces públicas del sistema, se rechaza;
2. Esté dentro de su periodo de validez — los certificados caducados o aún no válidos se rechazan;
3. Lleve una SAN que coincida con el host de la `Relay Address` a la que se conecta — un certificado emitido para otros nombres se rechaza.

Un límite a tener en cuenta: solo se verifica la conexión aguas arriba con el relay. Las reconexiones de Terminal y de solicitudes mantienen la política de certificados existente.

Si alguna comprobación falla, la conexión se rechaza y se reintenta en segundo plano con demoras crecientes; el servidor compartido queda inaccesible a través del relay hasta que la comprobación pase o usted desactive el modo estricto.

## Fallos habituales

- `La verificación estricta de TLS requiere un PEM de CA raíz` — el conmutador está activado pero el campo de la CA está vacío. Pegue primero el PEM;
- `Discordancia de SAN` — está llamando al relay por un nombre o IP que no está en su certificado. Añádalo en la página `Certificados y TLS` del relay (vea la página Dominios y gestión de SAN) y reemita;
- `Certificado caducado o aún no válido` — el reloj del relay está mal o el certificado hoja no pudo renovarse. Compruebe el estado del certificado en el relay;
- `No firmado por la CA configurada` — la CA del campo pertenece a otro relay, o la conexión está siendo interceptada. Compare las huellas antes de continuar.
