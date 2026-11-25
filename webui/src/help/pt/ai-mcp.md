O servidor traz um host do Model Context Protocol integrado, e a seção `MCP` da Web UI o configura: Code Agents (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) podem ligar-se a partir do seu próprio chat e trabalhar diretamente na sua máquina — ler e comandar terminais e até operar o EasyEDA Pro.

## Configuração do servidor

Ative o servidor MCP e configure:

- `Port` (padrão `6180`) e `Bind Address` — `127.0.0.1` mantém o endpoint só em localhost; `0.0.0.0` abre-o à rede;
- `Access Token` — exigido para pedidos que não venham do localhost: chamadores remotos devem apresentá-lo como Bearer token, enquanto pedidos do localhost são sempre admitidos. Nunca fica vazio — uma instalação nova traz um aleatório; o botão de dado regenera e o de copiar entrega;
- `Stream Throttle` — o intervalo mínimo entre envios de fluxo de tela para os agentes conectados.

As mudanças aplicam-se ao `Save`; o servidor recarrega a quente sem reiniciar.

## Endpoints

Cada conjunto de capacidades é servido na própria URL, `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — a superfície de terminal embutida. Os agentes podem listar terminais, ler a tela visível e o histórico, digitar entrada bruta e teclas nomeadas, redimensionar e esperar o prompt. As chamadas miram um terminal pelo seu Terminal ID — clique direito num terminal → `MCP` → `Copy Terminal ID` e cole o ID no contexto do agente (ver Terminal MCP na seção Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — um addon MCP embutido que faz a ponte do EasyEDA Pro para os agentes: um pequeno conjunto fixo de ferramentas mais um catálogo pesquisável de chamadas `eda.*`, encaminhadas à extensão conectora do EasyEDA por WebSocket. Ative primeiro o addon na seção `Addons` — o endpoint dele aparece na lista assim que instalado, e o emblema mostra se o daemon do addon está conectado. O campo `Timeout` do addon limita cada pedido ponte (aumente para operações EDA lentas);
- **Outros addons MCP** — qualquer addon do tipo MCP adiciona o seu endpoint da mesma forma, cada um com emblema conectado/desconectado ao vivo.

Escolha a sua CLI de agente no menu suspenso acima da lista; o botão de copiar ao lado de um endpoint entrega o comando de configuração pronto a executar.

## Conectar um agente

O token nunca precisa ser colado à mão. O servidor mantém um perfil de ambiente `Vibe Terminal MCP Bearer Token` gerado automaticamente (ver a seção Environment Variables) que exporta `VTERM_MCP_TOKEN`; aplique-o ao terminal antes de conectar o agente e use depois o comando copiado:

- **Apps de comando de shell** (Claude Code, Codex) — execute o comando `mcp add` copiado num terminal comum. Codex lê o token de `VTERM_MCP_TOKEN` via `--bearer-token-env-var`, portanto nenhum segredo acaba na linha de comando;
- **Apps de conversa** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — inicie o app com o perfil aplicado e envie o prompt copiado dentro da conversa. O agente atualiza diretamente sua configuração MCP. Kimi usa `bearerTokenEnvVar`; Pi, OpenCode e Qwen Code usam o token Bearer de `VTERM_MCP_TOKEN`, portanto o token nunca aparece no chat.

Dentro do Pi e do Qwen Code, `/mcp` inspeciona conexões.

O cartão `Command Templates` abaixo das configurações guarda os comandos por aplicativo: edite uma entrada, sobrescreva uma embutida ou adicione modelos para novas CLIs — a pré-visualização mostra exatamente o que o botão de copiar produzirá.
