El analizador de terminal del lado servidor vigila la salida del terminal: detecta avisos de comandos, reconoce la finalización de tareas y, con un LLM, clasifica lo que muestra el terminal. `Auto Confirm` es su función más usada — cuando el terminal se detiene en una confirmación ordinaria (como `Proceed? [Y/n]`), se envía Enter por ti, de modo que las tareas largas ya no se atascan en preguntas.

## Configuración

La sección `Terminal Analysis` de la interfaz web:

- `Terminal observer` — el interruptor principal del análisis del lado servidor;
- `Auto confirm` — envía Enter solo para avisos clasificados como **confirmaciones ordinarias**; las demás preguntas nunca se responden a ciegas;
- `Analysis delay (seconds)` — cuánto esperar tras el reposo de un agente antes de analizar, evitando llamadas LLM redundantes entre tareas;
- `Confirmation grace (seconds)` — la ventana durante la cual puedes tomar el control después de detectarse una confirmación;
- `Notify on completion` — emite un evento cuando la salida se clasifica como tarea terminada (conéctalo a notificaciones).

> La nota de la página: cuando el modo automático de tu propio agente está activado, conviene desactivar `Auto Confirm` en el cliente para que las dos automatizaciones no se respondan una sobre otra.

## Dependencia del LLM

La clasificación la hace un LLM: elige un `Provider` y rellena sus credenciales en la sección `LLM` (compartida por la clasificación de terminal, los mensajes de commit de Git, el asistente de chat y otras funciones de IA). Sin LLM el analizador sigue funcionando con precisión reducida — mantén la confirmación automática desactivada en ese caso.

![La página de ajustes Terminal Analysis](images/terminal-auto-confirm.jpg)

![La página de ajustes LLM — elige un proveedor y rellena credenciales](images/terminal-auto-confirm-llm.jpg)
