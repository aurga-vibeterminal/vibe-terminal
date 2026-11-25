El certificado hoja solo es válido para los nombres listados en sus **Nombres Alternativos de Sujeto (SAN)**. Cuando un cliente se conecta, el nombre de host o la IP de su URL debe coincidir exactamente con una de esas entradas — no hay soporte para comodines ni para coincidencias por prefijo. Esta página cubre la gestión de esa lista en el servidor; la comprobación en el lado del cliente está en la última página, Verificación estricta en clientes.

## Detección automática

El servidor descubre de forma continua los nombres por los que es alcanzable — los nombres de host actuales y las direcciones de las interfaces de red activas — y los lista bajo `Detectadas automáticamente`. Estas entradas siguen a la máquina y no se pueden editar a mano: añada una interfaz de red o cambie el nombre de host y la lista se actualizará en la próxima renovación.

## Dominios y direcciones IP adicionales

La detección no lo sabe todo: los nombres DDNS, las entradas de DNS de horizonte dividido o una dirección detrás de un NAT deben listarse a mano. Añádalos bajo `Dominios y direcciones IP adicionales`:

1. Escriba un nombre de host o dirección IP (por ejemplo `server.home.arpa` o `10.0.0.8`) y pulse `Añadir`;
2. Repita para cada nombre que vayan a usar los clientes. Elimine una entrada con el botón de la papelera;
3. Revise la `Vista previa final de SAN` — la vista combinada de las entradas detectadas y manuales que llevará el próximo certificado;
4. Pulse `Guardar y reemitir`. Al guardar se firma un nuevo certificado hoja de 90 días con la nueva lista de SAN y se recarga en caliente — las conexiones existentes siguen activas y los nuevos protocolos de enlace usan el nuevo certificado de inmediato.

## Reemisión manual y estado

El panel `Certificado hoja del servidor` muestra el certificado activo: una insignia de estado, el sujeto, la huella SHA-256, el periodo de validez y las entradas SAN. La insignia de estado indica:

- `Disponible` — hay un certificado hoja válido cargado y en uso;
- `Error al actualizar` — el último intento de reemisión falló; el certificado anterior permanece activo y el error se muestra en la página;
- `Sin generar` — aún no existe CA ni certificado hoja; use `Generar CA y certificado` para crear ambos.

`Reemitir certificado hoja` firma un nuevo certificado hoja de inmediato, sin esperar a un disparador automático — útil justo después de cambiar el nombre de host o las interfaces, o para recoger ediciones de SAN hechas fuera de la interfaz web.

## Distribuir la CA raíz

Los clientes que vayan a usar la verificación estricta necesitan el certificado público de la CA raíz. Consígalo desde la tarjeta `CA raíz privada`:

- `Descargar CA raíz` guarda el archivo PEM;
- `Copiar CA raíz` pone el PEM en el portapapeles, listo para pegar en el campo de CA de un cliente.

La CA nunca cambia entre renovaciones del certificado hoja ni ediciones de SAN — basta con distribuirla una vez. Antes de activar la verificación estricta en cualquier lugar, compare la huella SHA-256 del certificado mostrada en esta página con lo que recibió el cliente (vea la última página, Verificación estricta en clientes).

> La descarga y el portapapeles contienen solo el certificado público de la CA — nunca una clave privada. Entréguela únicamente a personas a quienes quiera permitir verificar su servidor.
