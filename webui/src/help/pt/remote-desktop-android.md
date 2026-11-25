Além das telas de desktop, a área de trabalho remota pode transmitir dispositivos Android — telefones reais por adb e emuladores AVD. O cliente os mostra numa janela com moldura de telefone com entrada por toque, rotação e controle remoto do ciclo de vida do emulador.

## Pré-requisitos

O servidor precisa encontrar a toolchain do Android:

- **SDK do Android** (fornece `adb`) — necessário para dispositivos e emuladores;
- **Android Studio** — necessário para inicializar emuladores AVD.

Ambos são autodetectados por padrão. Se a detecção falhar ou as ferramentas viverem fora dos locais padrão, ajuste `Android SDK directory` / `Android Studio directory` sob `Remote Desktop` → `Screen streaming tools`; as mudanças têm efeito imediato (aplicadas a quente, sem reiniciar o servidor).

## Fontes de dispositivos

Com os caminhos prontos, o servidor enumera automaticamente as fontes Android disponíveis:

- **Dispositivos reais conectados** — conecte por USB com a depuração USB ativada; qualquer coisa visível a `adb devices` pode transmitir;
- **Emuladores AVD** — emuladores em execução aparecem sozinhos; você também pode inicializar um AVD escolhido sob demanda do cliente (sem precisar iniciá-lo à mão antes) e desligá-lo remotamente.

Cada fonte ganha sua própria entrada na barra SCREENS do cliente.

## Orientação

O botão de rotação da janela do emulador define o `user_rotation` do sistema por `adb` — a direção de exibição do próprio dispositivo muda, e a janela se reajusta à nova resolução, adaptando-se tanto ao retrato quanto à paisagem.

## Lado do cliente

Abra a aba `Debug` no painel esquerdo do cliente de desktop e clique no botão `Android Simulator` da barra SCREENS (só aparece quando o servidor expõe fontes Android) para abrir a janela do emulador; o seletor de dispositivo muda a aparência da moldura e `Rotate` inverte a orientação. Veja Usar o cliente.

> A injeção de toque em dispositivos reais usa o canal de entrada do adb (Android 4.4+); algumas ROMs customizadas pelo fabricante a restringem, caso em que o dispositivo é somente visualização.
