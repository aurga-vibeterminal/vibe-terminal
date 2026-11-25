O Vibe Terminal é um hub de terminais gerenciado remotamente para desenvolvimento embarcado e de servidores. Um **servidor** roda em cada máquina em que você trabalha — uma máquina de desenvolvimento, um servidor de build, uma placa embarcada — e os **clientes** se conectam a ele por um aplicativo de desktop ou pela linha de comando. Este guia cobre os dois lados: a interface Web, que configura um servidor, e os clientes que trabalham com ele.

## Como tudo se encaixa

- Tudo roda no **servidor** — as sessões de terminal vivem lá, então fechar um cliente, perder a rede ou reorganizar janelas nunca interrompe um comando em execução; reconecte e continue de onde parou;
- Os clientes discam uma única conexão WebSocket criptografada (`wss://host:port`; `wss://localhost:5153` para o servidor local) e se autenticam com a **chave secreta** do servidor; o pareamento entrega a chave por um QR code ou por uma string de compartilhamento criptografada (veja Conectar um cliente);
- A **interface Web** é a própria página de configuração do servidor — perfis, addons, provedores de LLM, relay, encaminhamento de portas e mais. Não é um terminal: o trabalho do dia a dia acontece no cliente de desktop ou no `vtermc`.

## Componentes

- **Servidor** — roda na máquina de destino; hospeda as sessões de terminal e fornece arquivos, Git, área de trabalho remota, voz e recursos de IA aos clientes conectados;
- **Cliente de desktop** — o espaço de trabalho nativo: terminais com divisões, grupos de abas e janelas destacadas, um explorador de arquivos e editor, ferramentas Git, entrada de voz, histórico de conversa do agente de código, área de trabalho remota e depuração remota;
- **`vtermc` CLI** — inicie e anexe a sessões a partir de qualquer terminal simples, por SSH ou numa máquina headless; compartilha sua lista de servidores com o cliente de desktop (veja Cliente de linha de comando);
- **Interface Web** — configurações do servidor no navegador, organizadas em seções na barra lateral esquerda (veja Usar a interface Web).

## Ícone da bandeja

Em máquinas com uma sessão de área de trabalho, o servidor exibe um ícone na bandeja enquanto roda:

![Menu do ícone da bandeja — Open traz um cliente em execução para o primeiro plano ou inicia um, Settings abre a interface Web, e o item de voz inicia ou para o servidor ASR](images/start-tray-menu.jpg)

- **Open** — traz um cliente de desktop em execução para o primeiro plano, ou inicia um quando nenhum está rodando (clicar duas vezes no ícone faz o mesmo);
- **Settings** — abre a interface Web do servidor no navegador (veja Usar a interface Web);
- **Start/Stop Vibe Voice** — inicia ou para o servidor ASR sem abrir a interface Web, espelhando os botões Iniciar/Parar da seção `Voz`; aparece quando o reconhecimento de voz está habilitado, e o rótulo acompanha o estado (p. ex. `Stop Vibe Voice (model on device)` enquanto o modelo está carregado);
- **Exit** — desliga o servidor.

Enquanto pastas são hospedadas via HTTP, uma entrada `HTTP Servers` oferece parada com um clique para cada compartilhamento (veja Publicar com servidor HTTP).

## Mapa de recursos

O restante deste guia espelha os capítulos da barra lateral:

- **Terminais** — criar, dividir e organizar terminais; perfis, WSL e serial UART; comandos rápidos e variáveis de ambiente; controlar terminais a partir de agentes externos com MCP; revisar conversas do agente de código; confirmação automática de prompts interativos;
- **Arquivos** — projetos recentes, o explorador de arquivos do lado do servidor, enviar e baixar, e publicar um diretório via HTTP (veja Explorador de arquivos, Enviar e baixar e Publicar com servidor HTTP);
- **Git** — staging, commits assistidos por IA, worktrees, rebase interativo, resolução de conflitos e push;
- **Área de trabalho remota e simuladores** — assistir e controlar as telas do servidor, dispositivos Android e emuladores, e o simulador de iOS;
- **Entrada de voz** — reconhecimento de fala do lado do servidor com ditado por push-to-talk e polimento por LLM;
- **Relay** — alcançar máquinas atrás de NAT ou IPs dinâmicos por um único ponto de entrada público;
- **Acesso remoto** — domínios DDNS e encaminhamento de portas TLS para serviços rodando no servidor;
- **Recursos de IA** — provedores de LLM como addons, atribuídos por recurso, além da integração MCP (veja LLM e MCP).

## Por onde começar

1. Abra a seção `Server` da interface Web, defina uma `Secret Key` e pareie um cliente — Conectar um cliente mostra o passo a passo do QR code e da configuração manual;
2. Navegue pelas seções da interface Web para configurar perfis, addons e recursos — veja Usar a interface Web;
3. Abra um terminal no servidor e percorra os capítulos acima conforme necessário.
