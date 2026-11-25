Por defecto, un cliente cifra cada conexión pero no verifica quién está al otro lado — la misma postura de «confianza en el primer uso» que aceptar un certificado autofirmado en un navegador. La **verificación estricta** cambia eso: el cliente comprueba el certificado del servidor contra una CA raíz que usted proporciona y se niega a conectarse a menos que la cadena, la validez y las SAN pasen todas las comprobaciones. Esto hace que los ataques de intermediario sobre la conexión sean prácticamente imposibles, al precio de un paso manual: entregar al cliente la CA raíz del servidor.

## Dónde se configura

La verificación estricta se configura por servidor en el panel de edición de servidores del cliente, junto a la dirección y la clave secreta:

1. `Certificado de CA raíz (PEM)` — un campo multilínea para el certificado público de la CA raíz del servidor;
2. `Verificación estricta de la CA (cadena, validez y SAN)` — solo se puede marcar una vez introducida una CA.

![El panel de edición de servidores en el cliente de escritorio: campo multilínea de Certificado de CA raíz (PEM) con la casilla de verificación estricta debajo](images/client-ca-fields.jpg)

## Llevar la CA raíz al cliente

Dos formas, desde la misma fuente (la página `Certificados y TLS` del servidor — vea la página Dominios y gestión de SAN):

- **Copiar y pegar** — pulse `Copiar CA raíz` en la interfaz web y pegue el PEM en el campo `Certificado de CA raíz (PEM)` del cliente;
- **Importar una cadena de compartir** — las cadenas de compartir que produce el cliente incorporan la CA raíz junto con la dirección y la clave. Al importar esa cadena se rellena automáticamente el campo de la CA, y quien la recibe solo tiene que marcar la casilla de verificación estricta.

La CA es la misma para todos los clientes y nunca cambia entre renovaciones del certificado — basta con configurarla una vez por servidor.

## Qué comprueba el modo estricto

Con la casilla activada, cada conexión a ese servidor debe presentar un certificado que:

1. Encadene con la CA raíz configurada — un certificado firmado por cualquier otra cosa se rechaza;
2. Esté dentro de su periodo de validez — los certificados caducados o aún no válidos se rechazan;
3. Lleve una SAN que coincida con la dirección a la que se conecta — un certificado emitido para otros nombres se rechaza.

Si alguna comprobación falla, la conexión se rechaza con un error que indica qué comprobación falló. Sin la casilla (o sin una CA), las conexiones siguen cifradas pero se omiten estas comprobaciones.

## Verificar la huella

Antes de confiar en la CA, confirme que tiene la auténtica: la página `Certificados y TLS` muestra la huella SHA-256 de la CA raíz. Compárela con el PEM que está a punto de pegar (o con el PEM incorporado en una cadena que haya recibido). Si difieren, no active la verificación estricta — consiga la CA por un canal de confianza en su lugar.

## Fallos habituales

- `Discordancia de SAN` — se está conectando por un nombre o IP que no está en el certificado. Añádalo en el servidor (vea la página Dominios y gestión de SAN) y reemita;
- `Certificado caducado o aún no válido` — el reloj del servidor está mal o el certificado hoja no pudo renovarse. Compruebe el estado del certificado en la interfaz web;
- `No firmado por la CA configurada` — la CA del campo pertenece a otro servidor, o la conexión está siendo interceptada. Compare las huellas antes de continuar.
