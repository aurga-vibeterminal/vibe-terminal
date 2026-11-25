A área de trabalho remota permite assistir e controlar as telas do servidor a partir do cliente de desktop em tempo real: a área de trabalho da máquina é transmitida para uma janela do seu lado, com seu mouse e teclado injetados — como se você estivesse sentado na frente dela. Além das telas de desktop, emuladores Android e simuladores iOS podem ser vistos e dirigidos dentro de janelas com moldura de telefone.

![Cliente de desktop — uma janela de área de trabalho remota usada junto com terminais](images/remote-desktop-overview-hero.webm)

## O que ela faz

- **Telas de desktop** — cada tela física do servidor é transmitida para sua própria janela de cliente redimensionável;
- **Dispositivos / emuladores Android** — dispositivos conectados e emuladores AVD aparecem como fontes, visíveis e tocáveis numa janela com moldura de telefone, com rotação e inicialização/desligamento remoto do emulador;
- **Simulador iOS** (servidores macOS) — ver o simulador iOS do Xcode numa janela com moldura de telefone com seletor de dispositivo e rotação.

## Como funciona

O servidor captura a tela e a codifica com codificador, taxa de quadros, teto de resolução e taxa de bits configuráveis; o cliente decodifica em janelas flutuantes e envia os eventos de entrada de volta pela conexão criptografada cliente-servidor existente — sem portas extras.

A preparação por plataforma difere: o Windows pode rodar o servidor de tela como serviço do sistema e precisa de um driver de exibição virtual em máquinas headless; o Linux usa um consentimento Portal mais desbloqueio de sessão; o macOS exige permissão de Gravação de Tela. Veja Configuração do servidor; dispositivos Android são cobertos em Dispositivos Android, e as janelas do cliente em Usar o cliente.
