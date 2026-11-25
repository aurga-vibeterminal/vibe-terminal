`vtermc` é o cliente de linha de comando: conecta-se a um servidor, cria ou anexa sessões de terminal e gerencia a lista de servidores salva — tudo dentro de um terminal simples, por SSH ou numa máquina headless. Compartilha `~/.vibeterminal/client.json` com o cliente de desktop, então ambos veem os mesmos servidores.

## Iniciar um terminal

Execute `vtermc` sem sinalizadores de modo para criar um terminal novo no servidor local (`wss://localhost:5153` por padrão) com o shell detectado, e transmiti-lo para o terminal atual. Digite `exit` (ou `Ctrl+D`) no shell remoto para encerrar a sessão.

As sessões vivem no servidor: se a conexão cair — perda de rede, fechar a janela, matar o `vtermc` — a sessão continua rodando e pode ser reanexada depois com `vtermc -s`.

- `vtermc -p <id>` — iniciar com um perfil de terminal específico (a lista de perfis está na seção `Profiles` da interface Web);
- `vtermc -p` — sem id de perfil: o seletor lista os perfis do servidor interativamente;
- `vtermc -w <dir>` — definir o diretório de trabalho da nova sessão.

Quando stdin não é um terminal (um pipe ou arquivo), `vtermc` muda para o modo em lote: stdin é alimentado à sessão e a saída flui até a sessão fechar — útil a partir de scripts ou agentes:

```sh
echo "make test" | vtermc
```

## Conectar a um servidor

- `--server wss://host:port` — conectar a um servidor específico em vez do local (endereços IPv6 entre colchetes: `wss://[fd00::1]:5153`); a variável de ambiente `VTERM_SERVER` define o mesmo padrão;
- O handshake é respondido com a chave secreta do servidor: servidores adicionados via `-a`/`-s` carregam sua chave em `client.json`; o servidor local recorre à variável de ambiente `TERMINAL_GUI_SECRET_KEY` (ou uma chave em texto plano em `server.json`).

## Adicionar servidores

`-a` salva um servidor em `client.json` e sai — nunca abre um terminal. Duas formas:

- `vtermc -a` — prompts interativos: `Server name`, `Server address` e `Secret key`. O endereço deve ser `wss://host:port` (entrada inválida é pedida de novo; um endereço vazio cancela). Um nome vazio recorre ao endereço;
- `vtermc -a "vtshare1:..."` — cole a string de compartilhamento criptografada. Clicar no `QR Code` de pareamento na seção `Server` da interface Web a copia para a área de transferência; `vtermc` a descriptografa e preenche nome, endereço e chave secreta automaticamente.

Detalhes que valem saber:

- readicionar um endereço existente atualiza aquela entrada no lugar (seu id é mantido) em vez de duplicá-la;
- endereços locais são recusados — `Local` está sempre implicitamente disponível e nunca precisa de entrada;
- as entradas também são mostradas pelo Server Manager do cliente de desktop.

## Selecionar servidores e sessões

`vtermc -s` abre dois seletores em tela cheia (requer terminal interativo):

1. **Lista de servidores** — primeiro `Local`, depois cada servidor gerenciado como `Name  (endereço)`. Navegue com as setas ou `j`/`k`, salte com um dígito, `Enter` seleciona, `q` cancela;
2. **Lista de terminais** — primeiro `New Terminal`, seguido das sessões ativas daquele servidor com diretório de trabalho, título e servidor proprietário. Anexe a uma sessão existente, ou crie uma nova (que então honra `-p` / `-w` como uma execução simples de `vtermc`).

Anexar nunca interrompe a sessão — ela continua rodando no servidor entre anexações.

### Remover um servidor

Na lista de servidores, mova até um servidor gerenciado e pressione `d`: a linha fica vermelha e a dica pede confirmação. Pressione `y` (ou `Enter`) para excluir a entrada de `client.json`, qualquer outra tecla aborta. `Local` não pode ser excluído.

## Variáveis de ambiente

Uma nova sessão herda um conjunto fixo do seu shell — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — lista separada por vírgulas (ou a variável de ambiente `VTERM_ENV_INHERIT`); curingas `*` são suportados;
- `-e VAR=VALUE` — injetar variáveis extras (repetível);
- `--no-env` — iniciar a sessão sem nenhuma variável herdada.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Todas as opções

```text
-p, --profile [<ID>]          Selecionar perfil de terminal (omitir para listar interativamente)
-w, --working-dir <DIR>       Definir diretório de trabalho
-s, --select                  Selecionar servidor e terminal existente interativamente
-a, --add [<SHARE>]           Adicionar um servidor (vtshare1: string de compartilhamento, ou interativo)
    --server <URL>            URL WebSocket do servidor [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Variáveis de ambiente adicionais
    --no-env                  Não herdar variáveis de ambiente
    --env-inherit <PATTERNS>  Padrões de variáveis de ambiente a herdar [env: VTERM_ENV_INHERIT]
```
