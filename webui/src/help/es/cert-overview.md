El servidor protege todos los puntos finales TLS que expone — la interfaz web, las conexiones de clientes y los puertos reenviados — con un certificado emitido por su propia **CA raíz privada**. La página `Certificados y TLS` de la interfaz web es donde gestiona esa CA, controla para qué nombres es válido el certificado y entrega la CA a los clientes que quieran verificar el servidor estrictamente.

## Cómo funciona

- El servidor genera una CA raíz privada una sola vez y la conserva durante toda la vida de la instalación. Todo lo firma ella;
- La CA firma un **certificado hoja** para el servidor con una validez de 90 días, que cubre los nombres de host y direcciones IP que los clientes usan realmente para llegar al servidor;
- Cuando cambian los nombres de host, las interfaces de red o las entradas SAN de la máquina, el certificado hoja se reemite automáticamente — la CA raíz nunca cambia, por lo que los clientes que ya confían en ella siguen confiando en el servidor;
- El nuevo certificado hoja se sustituye a través de un resolutor de certificados compartido y se recarga en caliente: las conexiones existentes nunca se interrumpen y los nuevos protocolos de enlace usan de inmediato el certificado más reciente. Si una renovación falla, el certificado anterior permanece activo.

![La página Certificados y TLS: arriba la tarjeta de CA raíz privada, en el medio la gestión de SAN, abajo los detalles del certificado hoja del servidor](images/certificates-page.jpg)

## Qué encontrará en la página

1. `CA raíz privada` — descargue o copie el certificado público de la CA para distribuirlo a los clientes (y a sus navegadores). La descarga nunca contiene una clave privada;
2. `Dominios y direcciones IP (SAN)` — la lista de nombres para los que es válido el certificado hoja. Descrita en la página siguiente, Dominios y gestión de SAN;
3. `Certificado hoja del servidor` — el certificado activo: estado, sujeto, huella SHA-256, periodo de validez y entradas SAN.

## Certificados TLS personalizados

Si hay un certificado TLS personalizado configurado para el servidor, ese sigue mandando: el certificado gestionado de esta página se sigue generando y manteniendo al día, pero no reemplaza la identidad personalizada configurada. Mientras haya una identidad personalizada activa, la página muestra un aviso.

## Por qué confiar en la CA raíz

Los clientes que solo necesitan cifrado funcionan sin configuración adicional. Los clientes que quieren *verificar* el servidor — el modo estricto descrito en la última página, Verificación estricta en clientes — necesitan el certificado público de la CA raíz, y este debe llegar por un canal de confianza:

> Descargue la CA raíz solo desde la interfaz web local de este servidor o a través de un túnel de confianza, y verifique su huella SHA-256 (mostrada en esta página) antes de activar la verificación estricta en cualquier lugar.

El mismo CA firma también los certificados que usa el reenvío de puertos (vea la página Reenvío de puertos). Una vez que un dispositivo confía en la CA raíz, las URL reenviadas se abren sin la advertencia del navegador de *la conexión no es privada*.
