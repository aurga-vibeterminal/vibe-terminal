Pulsa `Ctrl` dos veces seguidas rápido (en menos de ~300 ms) en cualquier parte del cliente de escritorio para abrir el diálogo de entrada de voz.

## Grabación

El diálogo empieza a grabar de inmediato: la forma de onda líquida sigue tu voz, el selector superior cambia de micrófono (la elección se recuerda), y un temporizador muestra el tiempo transcurrido. `Stop` termina el clip y lo envía a transcripción (`Reset` reinicia).

![La fase de grabación — forma de onda, selector de micro y temporizador](images/voice-client-record.jpg)

## Procesamiento

Una animación de flujo de partículas se reproduce mientras el servidor transcribe el audio.

## Edición

La transcripción aterriza en un cuadro de texto editable:

- la pastilla de modo alterna entre `original` y `polished`; `Polish` ejecuta el LLM una vez, y `Auto polish` lo hace justo después de cada transcripción;
- `Re-record` descarta el texto y empieza de nuevo; `Insert` graba otra vez e inserta la nueva transcripción en el texto existente en el cursor (reemplazando cualquier selección), para que extiendas o corrijas lo ya dictado — el cursor queda justo después del fragmento insertado;
- pasa el puntero sobre el cuadro de texto para revelar un botón de copia en su esquina superior derecha: un clic copia el texto y se convierte en una marca; editar el texto o cambiar de modo lo rearma para otra copia;
- `Submit` (o `Ctrl+Enter`) teclea el texto en el terminal activo; `Esc` cierra el diálogo.

![La fase de edición — pastilla de modo, pulido automático, la fila de acciones con Insert, y el botón de copia al pasar el puntero](images/voice-client-editing.jpg)

## Configuración del pulido

`Polish` necesita un LLM de pulido de voz: en la sección `LLM` de la interfaz web, activa `Enable voice text polish` en la lista por función (un proveedor normal más este conmutador es todo lo que hace falta). Sin él el botón `Polish` permanece atenuado y la opción de pulido automático queda oculta.
