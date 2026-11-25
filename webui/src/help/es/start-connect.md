Conecta un cliente a este servidor con el código QR de emparejamiento de la interfaz web — o escribiendo los datos a mano. Todo lo que un cliente necesita (dirección, puerto, clave secreta) se muestra en la sección Server.

## Preparar el servidor

- Abre la sección `Server` y asegúrate de que haya una `Secret Key` establecida; pulsa el botón de dado para generar una. Regenerar la clave invalida cada cliente emparejado, que deberá emparejarse de nuevo.
- Despliega el bloque `QR Code` y elige la `Network Interface` y la `IP Version` (IPv4 / IPv6) que las demás máquinas usarán para alcanzar este servidor.
- Si hay un dominio DDNS configurado, puedes cambiar `Connection Type` a `DDNS Domain` para que el código QR incruste el dominio en lugar de una dirección IP.
- La `Pairing Preview` muestra la URL de conexión exacta (`wss://host:port`) que marcará el cliente.
- Pulsar el `QR code` copia una cadena de uso compartido cifrada al portapapeles — los clientes la usan para el autorrelleno con un toque.

## Cliente de escritorio

- Pulsa el botón del nombre del servidor en la barra de título y elige `Servers` en el menú para abrir el diálogo Server Manager.

![Cliente de escritorio — el menú de servidor abierto al pulsar el botón del nombre del servidor](images/start-connect-server-menu.jpg)

- Pulsa el botón `+` junto a la lista de servidores para iniciar el formulario Add Server.
- Si la cadena de uso compartido está en el portapapeles, aparece un icono de portapapeles en la parte superior derecha del formulario — púlsalo para descifrar la cadena y autorrellenar `Title`, `Address` y `Secret Key`.
- Si no, rellena los campos a mano: un nombre para mostrar, la URL de conexión (`wss://host:port`, direcciones IPv6 entre corchetes) y la clave secreta.
- Pulsa `Save`, luego elige el nuevo servidor para abrir una sesión.

![Cliente de escritorio — Server Manager con el formulario Add Server](images/start-connect-add-server.jpg)
