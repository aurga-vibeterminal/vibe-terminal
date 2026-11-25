Serial UART transforma o console serial de um dispositivo (porta de depuração, placa embarcada, SBC) numa aba de terminal que você pode assistir e com a qual interagir como qualquer outro terminal.

## Configurar uma porta

Na seção `UART` da interface Web → `Add UART Port`:

- `Name` — mostrado como título do terminal (p. ex. «Debug Console»);
- `Serial Port` — escolha entre as portas detectadas (a contagem é mostrada no topo da página);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — corresponda ao firmware do dispositivo;
- `Terminal Type` e `Encoding` — a codificação é Auto (UTF-8) por padrão e pode seguir o firmware;
- `Show Advanced Options` expõe mais parâmetros seriais.

Portas salvas aparecem na lista `Configured UART Ports` e podem ser editadas ou removidas a qualquer momento.

![A página de ajustes UART — o formulário de porta](images/terminal-uart-config.jpg)

## Uso no cliente

Portas configuradas abrem como abas de terminal como sessões comuns do servidor:

- dados seriais são exibidos conforme chegam; a entrada do teclado vai direto para a porta;
- o menu de contexto oferece `Clear` (limpar a tela) e um submenu `Encoding` (trocar codificação ao vivo; a atual fica marcada);
- sessões UART não têm recursos de shell (sem Comandos rápidos, Apps ou itens de menu Envs).

Usos típicos: consoles de roteador/switch, logs de placas embarcadas, monitoramento de bancadas de flash.
