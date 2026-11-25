Quando você roda um agente de código (Claude Code, Codex, Kimi, Pi, OpenCode…) num terminal, a conversa dele fica presa naquela janela de terminal — rolada entre saídas de comandos, perdida assim que a tela avança. O `Histórico de conversa do agente de código` ergue a conversa completa para um painel dedicado: cada troca mostrada estruturada, conversas passadas comutáveis, mensagens traduzíveis — e você pode enviar comandos ao terminal direto do painel.

## Abrir o painel

Clique no botão de chat (`AI`) no lado direito da barra de título para alternar o painel Copilot. O menu suspenso no topo lista **todas as conversas passadas** do agente rodando na sessão de terminal ativa, mais recentes primeiro; escolher uma a carrega.

![O painel Copilot — o menu suspenso de conversas e o fluxo de mensagens](images/terminal-agent-history.jpg)

## Como funciona

- O servidor sabe qual aplicativo de agente roda no terminal e analisa seus **arquivos de sessão nativos** (o formato JSONL/histórico de cada agente sob seu diretório de projeto); mudanças no arquivo chegam ao vivo, mantendo a visão da conversa atualizada;
- Agentes suportados: `claude` / `codex` / `kimi` / `pi` / `opencode`; outros aplicativos ainda não têm painel de conversa;
- Mensagens carregam um papel (você / agente) e um tipo (texto / raciocínio **Thinking**), junto com o nome do modelo e a hora, renderizadas como Markdown.

## Recursos do painel

- **Trocar de conversa**: o menu suspenso `Conversations` no topo move entre sessões passadas, cada entrada mostra um resumo e sua hora;
- **Traduzir**: qualquer mensagem do agente pode ser traduzida sob demanda; a tradução substitui o texto no lugar (e pode ser alternada de volta);
- **Enviar comandos diretamente**: digite na caixa na parte inferior (Shift+Enter para nova linha) e pressione Enter — o comando vai direto para o terminal atual, como digitar na própria caixa de entrada do agente sem ocupar a visão do terminal.

## Encaixe com outros recursos

- O painel e o terminal são duas visões da mesma sessão: as perguntas do agente no terminal continuam respondidas lá (ou via Confirmação automática); o painel serve para revisar o histórico, não para interceptar a interação;
- Combinado com `Terminal MCP` (página anterior), agentes externos também podem ler a tela do terminal e executar comandos — a conversa mostrada aqui mais as ações MCP formam o registro completo do trabalho do agente.
