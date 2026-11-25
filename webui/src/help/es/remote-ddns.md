La banda ancha doméstica y de oficina suele venir con una IP pública dinámica que cambia sin aviso. El DDNS mantiene un dominio fijo apuntado a la IP actual de la máquina — clientes y otros servidores solo necesitan recordar el nombre, nunca la dirección.

## Cómo funciona

![Arquitectura DDNS — el gestor detecta periódicamente la IP local, el addon actualiza el registro DNS por la API del proveedor, los clientes conectan por dominio](diagram/ddns)

Toda la tubería se ejecuta automáticamente en segundo plano en el servidor:

1. El **gestor DDNS** se refresca en un ciclo fijo (cada 5 minutos por defecto, ajustable con la variable de entorno `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); guardar la configuración dispara un refresco inmediato, y nada se ejecuta mientras el servicio esté deshabilitado;
2. La **detección de IP local** lee primero la interfaz de red elegida, recurriendo a un descubrimiento basado en rutas, y filtra el resultado por el ajuste `IP Family` (IPv4 / IPv6 / dual);
3. El **addon DDNS** — un script JS aislado — recibe la configuración descifrada y la IP actual y llama a la API del proveedor DNS por HTTP. El addon Cloudflare integrado, por ejemplo, busca la zona por dominio y reescribe los registros A / AAAA a la IP actual, borrando registros obsoletos cuando desaparece una familia de direcciones;
4. Los **clientes** resuelven el dominio a la última IP y conectan por nombre.

## Configuración

Abre la sección `DDNS` de la interfaz web:

1. Activa `Enable`. También puedes dejar el servicio apagado mientras rellenas todo — un servicio deshabilitado guarda su configuración pero nunca se ejecuta;
2. elige un proveedor bajo `Provider`; `Cloudflare DDNS` viene integrado;
3. rellena los campos del proveedor (ejemplo Cloudflare):
   - `Email` — el correo de la cuenta de Cloudflare;
   - `API Token` — un token con permiso Zone → DNS → Edit;
   - `Domain` — el dominio raíz de la zona, p. ej. `example.com`;
   - `Subdomain` — el nombre del registro dentro de la zona, p. ej. `sub` para `sub.example.com`; déjalo vacío para la raíz de la zona;
4. en `Network Interface & IP`, elige la `Interface` y la `IP Family`. `Dual Stack` exige que la interfaz tenga dirección IPv4 e IPv6;
5. pulsa `Save`, luego `Sync Now` para verificar. `Sync Status` muestra la última IP enviada y la hora de sincronización, junto al mensaje de error del addon en caso de fallo.

## Seguridad

Las credenciales del proveedor (tokens de API) se cifran antes de almacenarse (`enc-v2:`) y solo se descifran en memoria mientras el addon se ejecuta; los addons corren en un runtime JS aislado cuyo único cabal es el HTTP saliente.

## Ampliar proveedores

Los proveedores son addons: un manifiesto JSON (que declara los campos de configuración que la interfaz web convierte en formulario) más un script JS que expone `run()`. El `cloudflare-ddns` integrado es la implementación de referencia — los proveedores personalizados pueden subirse como zip en la sección `Addons`.

## Combinación con otras funciones

- una vez el dominio resuelve, el código QR de la sección `Server` puede cambiar `Connection Type` a `DDNS Domain`, para que los clientes escaneen una conexión por dominio;
- otros servidores pueden poner `wss://your-domain:port` en su `Relay Address` para correr la cadena de relay sobre el dominio (véase el capítulo Relay).
