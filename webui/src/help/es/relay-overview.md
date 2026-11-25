El relay resuelve un problema: la mayoría de las máquinas están tras NAT o IP dinámicas, así que los clientes no pueden marcarlas directamente una a una. Deja que una máquina públicamente alcanzable actúe como punto de entrada del relay; cada otra máquina marca hacia ella. Un cliente mantiene entonces una única conexión al relay y alcanza a través de él los terminales de cada servidor compartido.

## Arquitectura

![Arquitectura del relay — cada cliente mantiene una conexión al relay; los servidores compartidos marcan al relay aguas arriba](diagram/relay)

- **Servidor relay** — el punto de entrada público al que se conectan clientes y servidores compartidos. Deja su `Relay Address` vacío.
- **Servidor compartido** — una máquina cuyo `Relay Address` apunta al relay. Marca aguas arriba y comparte sus terminales a través de él. La franja de color de cada nodo compartido en el diagrama refleja la franja que el cliente de escritorio pinta en las pestañas de ese servidor — un color por servidor.
- Las conexiones siempre se originan desde dentro: los servidores compartidos marcan al relay, así que no necesitan reenvío de puertos ni IP pública.

## Lo que ve el cliente

Conectado al relay, cada servidor compartido se comporta como una máquina local:

- las pestañas de terminal llevan franja de color e icono del SO remoto, identificando la máquina de un vistazo (véase la Guía de configuración);
- el menú de perfiles agrupa terminales por servidor — abre un terminal en cualquier máquina directamente;
- los grupos de pestañas pueden arrancarse a su propia ventana, dedicando una ventana a una máquina.

## Modelo de seguridad

Clientes y servidores compartidos se emparejan con la misma clave secreta; las conexiones se tunelan a través del relay. La clave viaja en el código QR de emparejamiento / cadena de uso compartido, nunca en texto plano.

Las instrucciones paso a paso están en la página siguiente, Guía de configuración.
