Tudo vive na seção `Remote Desktop` da interface Web; a página mostra os ajustes que se aplicam à plataforma do servidor.

## Chave principal

`Enable Remote Desktop` — uma vez ligado, clientes remotos conectados podem ver a área de trabalho desta máquina e enviar entrada.

## Ajustes por plataforma

**Windows**

- `Windows Service` — registra o servidor de tela como serviço de sistema de autoinício para que a área de trabalho remota sobreviva a reinicializações sem sessão logada. `Install` / `Uninstall` dispara um aviso UAC no servidor (o botão mostra progresso enquanto espera); a insígnia de status distingue em execução / instalado / não instalado.
- `Máquinas headless (sem monitor)` — sem uma tela conectada (ou acordada), o Windows não mantém área de trabalho capturável. A página aponta o download do driver de código aberto Virtual Display Driver (ARM64 / x64 escolhido automaticamente); instale-o para que uma área de trabalho virtual sempre exista.

![Interface Web — a seção Remote Desktop no Windows (cartões de serviço e headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — desbloqueia automaticamente a sessão travada quando uma transmissão inicia (onde o ambiente suporta);
- a captura de tela passa pelo Portal; o `Reset Permission` do cartão `Screen Recording Permission` limpa um consentimento salvo do Portal para você escolher outra tela;
- `Auto-start on Login` (na seção `Servidor`) — em hosts com gerenciador de display, o servidor inicia junto com a sessão gráfica, então a captura herda o ambiente de área de trabalho completo; em hosts headless, ele inicia no boot antes de qualquer login. Para alcançar a área de trabalho quando ninguém fez login localmente, ative também o login automático no gerenciador de display (GDM, SDDM, …); por segurança, o bloqueio automático de tela pode continuar ativado — a sessão e o servidor seguem em execução enquanto travada, e o `Unlock on Connect` (ou `loginctl unlock-session`) abre a sessão remotamente.
![Interface Web — o cartão Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — o sistema exige aprovação explícita para captura. Clique em `Grant Permission` para abrir o diálogo do sistema; `Reset Permission` recupera de um estado travado.

## Ferramentas de transmissão de tela (suporte Android)

`Android SDK directory` e `Android Studio directory` localizam o adb e o emulador; deixe vazias para autodetecção. Uma vez definidas, a página mostra o caminho resolvido e sua origem (auto / env / override); `Reset to Auto-Detect` restaura o padrão. Veja Dispositivos Android para o que elas habilitam.

## Codificação da transmissão

- `Encoder` — Auto ou um backend específico; o servidor lista codificadores utilizáveis e esmaece os indisponíveis com o motivo;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — teto do lado de captura (4K / 2.5K / 2K);
- `Bitrate` — `Auto` escala por área com a resolução (taxa base até 1080p, ~4× em 4K, limitada a 40 Mbps); um valor fixo aplica-se imediatamente às transmissões em execução, enquanto codificador / fps / resolução aplicam-se a transmissões iniciadas depois;
- o rodapé do cartão mostra o codificador / fps / resolução / taxa de bits vigentes.

![Interface Web — ajustes de codificação da transmissão](images/remote-desktop-server-stream.jpg)

Clique em `Save` para aplicar as mudanças.
