`Host with HTTP server` publica cualquier directorio del servidor como servicio de archivos HTTP con un clic: otros dispositivos de la misma red (o a través del relay) — un teléfono, otro ordenador, un banco de pruebas — pueden navegar y descargar ese directorio desde un navegador. Ideal para compartir artefactos de compilación y lotes de registros, o entregar archivos a dispositivos sin el cliente instalado.

## Cómo usarlo

Tres entradas, todas sobre un directorio: el menú contextual de la lista de archivos, el menú contextual de una entrada de Recent, y el menú contextual de las migas de pan → `Host with HTTP server`.

Al pulsarlo:

1. el servidor inicia un servicio HTTP estático enraizado en ese directorio, con el puerto tomado del rango de puertos Static HTTP;
2. **Servidor local**: la URL `http://127.0.0.1:port/` se copia directamente al portapapeles, con un mensaje de estado «HTTP server: … (copiado al portapapeles)»;
3. **Servidor remoto** (conectado a través de un relay): el cliente solicita automáticamente un mapeo de proxy inverso y copia la URL de uso compartido alcanzable desde fuera — las restricciones LAN no son un obstáculo.

Abre esa URL en el navegador de cualquier dispositivo y el directorio se navega y descarga como un listado web normal.

## Gestionar servicios en ejecución

Cada servicio HTTP sigue corriendo hasta detenerse:

- abre la sección `Static HTTP` de la interfaz web; la lista `Services` muestra cada servicio en ejecución y su raíz de documentos;
- cada entrada tiene un botón de parada; el rango de puertos se configura en la misma página.

> El servicio es un listado de archivos de solo lectura — sin puntos de escritura; el contenido del directorio es visible en vivo a medida que cambia, lo que conviene a registros y artefactos producidos continuamente.
