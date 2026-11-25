Comandos rápidos fixam os comandos build/test/deploy de um projeto no menu de contexto do terminal — um clique para executar, disponível em todo lugar. Os comandos são mantidos por repositório em `.vterm/commands.json` na raiz do repositório.

## Criar commands.json

Clique com o botão direito dentro de um terminal → `Commands`:

- quando o repositório da sessão ainda não tem commands.json, o menu oferece uma única entrada, `Generate commands.json` — clicá-la faz o servidor detectar o tipo de projeto (npm / Cargo / .NET / Xcode / Gradle-Android / genérico) e escrever um modelo inicial conservador. Um arquivo existente não vazio **nunca é sobrescrito**;
- localização do arquivo: `.vterm/commands.json` na raiz do repositório Git (o diretório de trabalho atual para pastas não-Git).

## Refinar

O modelo gerado contém apenas os comandos usuais do tipo de projeto detectado. Deixe seu agente de código (por exemplo um agente de editor conectado ao MCP deste servidor) ler o projeto e refinar o arquivo:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

Um array de nível superior também é aceito; `env` é opcional e define variáveis antes de o comando rodar; a chave legada `value` ainda é lida. O campo opcional `os` limita uma entrada ao sistema operacional em que o `vterm-server` roda: use `"win"` para Windows, `"linux"` para Linux ou `"osx"` para macOS. Se `os` for omitido, definido como `"all"` ou tiver um valor não reconhecido, a entrada fica disponível em todas as plataformas. O servidor vigia o CWD e mudanças no commands.json e as transmite — reabra o menu e os novos comandos estão lá.

## Executar

Botão direito → `Commands` → clique numa entrada para executá-la naquele terminal (o env opcional é aplicado antes). O menu fica indisponível enquanto um aplicativo de agente roda na sessão, para que texto de comando não caia dentro da interface do aplicativo.

## Exemplo: build e instalação Android por Wi-Fi

Um passo a passo completo num projeto Gradle-Android: fixar a configuração de ADB sem fio e o fluxo de build-instalação no menu de contexto, depois reconstruir o app com um clique.

**1. Passar o telefone para ADB sem fio.** Estes dois comandos são digitados uma vez por dispositivo — `adb tcpip 5555` enquanto o telefone está no USB, depois `adb connect` com o endereço do telefone:

![Terminal — passando um telefone Android para ADB sem fio com adb tcpip 5555 e adb connect](images/terminal-commands-adb-setup.gif)

**2. Deixar o agente de código escrever os comandos.** Abra o agente de código do projeto (aqui Claude Code) num terminal e peça para adicionar comandos de build e instalação — ele lê o layout do projeto e escreve as entradas em `.vterm/commands.json`:

![Terminal — Claude Code editando commands.json com os comandos de build e instalação do APK](images/terminal-commands-agent-edit.mp4)

**3. Construir e instalar pelo menu.** Divida o terminal para a saída do build ter seu próprio painel, depois botão direito → `Commands` → a entrada — o Gradle monta o APK e o instala no telefone por Wi-Fi:

![Terminal — dividindo um painel e executando o comando Android de build e instalação pelo menu de contexto](images/terminal-commands-split-run.mp4)
