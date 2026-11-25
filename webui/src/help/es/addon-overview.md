Los addons amplían el servidor con capacidades nuevas sin tocar el núcleo: proveedores LLM, backends de reconocimiento de voz, actualizadores de DNS dinámico, superficies de herramientas MCP adicionales y la detección de apps de agentes de código viven todos como addons. La sección `Addons` de la Web UI los gestiona.

## Categorías

Cada addon declara un tipo, y la sección agrupa las tarjetas en consecuencia:

- **Applications** — enseñan al observador de terminales a reconocer una CLI de agente de código (Claude Code, Codex, Kimi Code, Pi y OpenCode vienen integrados): detección de procesos, estado ocupado/inactivo, ganchos de automatización;
- **LLM Providers** — backends de completado de chat entre los que eligen las funciones de IA (un proveedor compatible con OpenAI viene integrado);
- **Voice / ASR** — backends de reconocimiento de voz (el addon integrado `vibe-voice` apunta al servidor ASR local supervisado);
- **Dynamic DNS** — mantienen un nombre de host apuntando a la dirección actual (Cloudflare viene integrado);
- **MCP** — superficies de herramientas MCP adicionales servidas en el host MCP (EasyEDA Pro viene integrado);
- cualquier otra cosa cae en el grupo **Other**.

## Addons integrados y subidos

- Los addons **integrados** van dentro del programa y siempre están disponibles. Su manifiesto y fuente permanecen prístinos — una fuente sobrescrita puede restaurarse en cualquier momento;
- Los addons **subidos** son tuyos: un zip con un manifiesto y una fuente JavaScript, instalado con el botón `Upload Addon`;
- La insignia `Modified` marca un addon cuya fuente has sobrescrito.

Cada tarjeta muestra el tipo, las insignias y un interruptor de activación; al expandirla aparecen la descripción, el autor, los campos de configuración y el código fuente.

![La sección Addons — tarjetas agrupadas con insignias e interruptores](images/addon-section.jpg)

## Configuración

El manifiesto de un addon declara sus campos de configuración, y la tarjeta expandida los convierte en un formulario. Los valores se guardan por addon y solo surten efecto mientras esté activado; los campos de tipo contraseña permanecen enmascarados. Desactivar un addon conserva su configuración: al reactivarlo, continúas donde lo dejaste.
