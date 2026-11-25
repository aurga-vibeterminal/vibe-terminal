Há duas formas de usar WSL no Windows: a leve adiciona um shell WSL como perfil de terminal, a completa instala o vterm-server dentro do WSL. A primeira só dá um shell; a segunda desbloqueia gerenciamento de arquivos, o painel Git, MCP e todo o resto.

## A forma leve: um perfil de terminal WSL

No Windows, **perfis de terminal WSL carregam automaticamente**: na inicialização o servidor detecta `wsl.exe` e enumera as distribuições instaladas, adicionando uma entrada de perfil por distro à lista (p. ex. `Ubuntu (WSL)`; uma entrada genérica `WSL` aparece quando nenhuma distro está listada).

Para usar uma, simplesmente escolha a entrada WSL no menu de perfis da barra de título do cliente — a distro inicia e o deixa direto no terminal dela, como abrir um local.

Limitação: essas sessões rodam no servidor do lado Windows — **o explorador de arquivos, o painel Git, as variáveis de ambiente e afins do cliente não funcionam** (operam com semântica do sistema de arquivos Windows e não veem o sistema de arquivos Linux dentro do WSL).

![O menu de perfis de terminal — as distros WSL listadas automaticamente](images/terminal-wsl-profile.jpg)

## A forma completa: instalar vterm-server dentro do WSL

Trate o WSL como sua própria máquina Linux e instale o vterm-server headless. O site oferece um script de uma linha — cole num terminal WSL:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

O script automaticamente:

1. Detecta o ambiente WSL/headless e baixa o vterm-server **headless** para `~/.local/vterm`;
2. Instala um **serviço de usuário systemd** e o inicia imediatamente — roda a cada login (início do WSL incluído), sem início manual;
3. Adiciona `~/.local/vterm` ao seu PATH.

> Quando o WSL tem systemd desabilitado, o script aponta adicionar `[boot]` `systemd=true` ao `/etc/wsl.conf`, executar `wsl --shutdown` no lado Windows e rodar o instalador de novo.

## Abrir a página de configuração

Dentro do WSL:

```bash
vterm-server --open
```

O comando imprime o endereço da interface Web (`http://localhost:<config-port>/`) e tenta abrir um navegador (no WSL via `wslview`, o navegador padrão do Windows). Graças ao encaminhamento de localhost do WSL2, **abrir o mesmo endereço num navegador Windows** funciona igualmente bem. Enquanto o servidor estiver rodando, o URL impresso carrega um token de acesso de uso único (`?ticket=...`, emitido via um socket local), então a página já abre autenticada.

## (Opcional) Compartilhar o servidor WSL no relay do host

Se a máquina Windows também roda vterm-server (atuando como relay), o servidor WSL pode se acoplar sob ele para o cliente ver as duas «máquinas» por uma única conexão local:

1. Na interface Web do servidor Windows, clique no QR code para copiar a string de compartilhamento;
2. Abra a interface Web do servidor WSL (endereço da seção anterior), clique no botão de colar que aparece ao lado do campo `Relay Address` para preencher automaticamente endereço e chave, depois `Apply`;
3. Enquanto o vterm-server do Windows roda, o cliente vê os terminais, arquivos e Git do servidor WSL sob o relay local.

> Sob o networking NAT padrão do WSL2, o WSL alcança o host Windows pelo endereço do host visto do lado WSL (o gateway padrão — o nameserver em `cat /etc/resolv.conf`); com networking espelhado `localhost` funciona. Veja o capítulo Relay para detalhes do relay.
