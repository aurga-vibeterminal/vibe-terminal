O cliente de desktop abre telas remotas pela barra SCREENS no topo do painel Debug. Cada tela vive na própria janela flutuante utilizável junto com terminais.

## Abrindo a barra SCREENS

1. Conecte-se ao servidor alvo (diretamente ou por um relay);
2. clique na aba `Debug` na barra de atividades esquerda (o ícone de ferramentas);
3. a barra `SCREENS` aparece no topo do painel: um botão de exibição por cada tela de desktop física do servidor, seguido das alternâncias `iOS Simulator` e `Android Simulator`.

![O painel Debug — a barra SCREENS com os botões de exibição e as alternâncias de simulador](images/remote-desktop-client-screens.jpg)

## Janelas de tela de desktop

Clique num botão de exibição para abrir a janela daquela tela:

- a janela é livremente redimensionável (até 800×600) e a imagem se reorganiza com ela;
- servidores multimonitor ganham uma janela por tela; várias podem estar abertas ao mesmo tempo;
- durante a conexão, a janela mostra um marcador «preparando área de trabalho»; se falta permissão de captura ao servidor, a janela mostra diretamente as instruções de permissão da plataforma;
- clique no botão da barra de novo ou feche a janela para parar de assistir (o estado ativo do botão permanece sincronizado com a janela).

## Janela do simulador iOS

Clique em `iOS Simulator` para abrir uma janela de moldura de telefone de tamanho fixo (264×640 retrato):

- o seletor de dispositivo no topo muda a aparência do telefone; a janela se reajusta à resolução escolhida;
- `Rotate` alterna retrato/paisagem (janela 524×380 em paisagem);
- requer um servidor macOS rodando o simulador iOS do Xcode.

## Janela do emulador Android

`Android Simulator` funciona do mesmo jeito: moldura de telefone fixa, seletor de dispositivo, botão de rotação. Fontes transmissíveis incluem emuladores AVD em execução e dispositivos reais conectados; um AVD também pode ser inicializado ou desligado sob demanda do cliente (veja Dispositivos Android).

> Janelas que transmitem um servidor localhost são automaticamente excluídas da captura de tela para evitar um espelho de realimentação; janelas que transmitem uma máquina remota permanecem capturáveis para que uma gravação externa ainda as mostre.
