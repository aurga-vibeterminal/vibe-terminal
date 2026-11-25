Cada terminal é um cidadão de primeira classe do MCP (Model Context Protocol): agentes externos dirigem terminais pelas ferramentas MCP do servidor — listar terminais, ler a tela, executar comandos, esperar a conclusão — permitindo que o agente de código que você já usa opere shells interativos reais.

## As ferramentas MCP de terminal do servidor

Em torno de sessões de terminal, um cliente MCP pode:

- `list_terminals` — todos os terminais abertos; `get_terminal` — título, cwd e estado de aplicativo de um terminal;
- `get_screen` — o texto visível atual da tela; `get_scrollback` — linhas recentes do histórico;
- `send_command` — executar uma linha de comando (Enter anexado); `send_input` — texto bruto sem Enter (confirmar `y`, preencher formulários); `send_key` — teclas nomeadas (Enter, Ctrl+C, setas);
- `clear_terminal` — limpar o buffer; `resize_terminal` — redimensionar o PTY; `wait_terminal` — bloquear até o terminal não estar mais ocupado.

## Endereçar um terminal: o Terminal ID

Chamadas de ferramenta miram um terminal pelo seu Terminal ID. Botão direito dentro de um terminal → `MCP` → `Copy Terminal ID`, depois cole o ID ao seu agente para que ele dirija exatamente aquele terminal.

![Botão direito do terminal — o submenu MCP](images/terminal-mcp-menu.jpg)

## Registro de aplicativo de agente com um clique

O submenu `MCP` também lista os aplicativos de agente detectados no servidor (CLIs de editor e afins). Clicar num aplicativo digita um comando `mcp add` pronto no terminal — **sem quebra de linha final**; revise-o e pressione Enter você mesmo para registrar o serviço MCP deste servidor com aquele agente. Entradas se escondem enquanto um aplicativo já roda na sessão, para que texto não caia dentro da interface dele.

> Ciclo típico: copiar o Terminal ID ao seu agente → `send_command` inicia um build → `wait_terminal` até terminar → `get_screen` lê o resultado. Junto com a Confirmação automática (veja o capítulo de terminais), tarefas interativas longas rodam quase sem supervisão.
