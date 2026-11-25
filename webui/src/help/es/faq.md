## El navegador avisa «La conexión no es privada» — ¿es normal?

Sí. Los puertos redirigidos, la interfaz web y las conexiones de los clientes comparten el mismo certificado autofirmado del servidor; el cifrado en sí no está comprometido. Importa el certificado del servidor en el almacén de confianza de tu dispositivo para eliminar el aviso de forma permanente.

## ¿Vibe Terminal es gratis?

Sí — el cliente/servidor de escritorio y el servidor headless son gratuitos. Descárgalos e instálalos en tantas máquinas como quieras.

## ¿A dónde van mis datos?

A ninguna parte donde no los hayas puesto. Las sesiones, las credenciales (enc-v2:) y el reconocimiento de voz viven en tus máquinas; las claves de emparejamiento nunca viajan en texto plano.

## ¿Puedo exponer un servicio redirigido de forma segura?

Sí. Los recursos integrados exigen token: los enlaces que el cliente genera para un recurso `Host with HTTP server` o la página de Settings incluyen un token de acceso, y una conexión sin token válido y sin sesión válida se corta antes de servir nada. En la primera visita, el token se canjea por una cookie de sesión segura específica de ese servicio y desaparece de la URL — un ticket de Settings funciona una sola vez y caduca al minuto, y detener o reiniciar un recurso invalida sus enlaces y sesiones anteriores. Trata el enlace completo como una credencial y entrégaselo solo a personas de confianza. Una redirección a tu propio servicio añade el TLS del certificado del servidor pero ninguna autenticación — mantén autenticación ante los servicios sensibles y desmonta una redirección al instante desde el panel de redirección de puertos cuando termines.
