A banda larga doméstica e de escritório tipicamente vem com um IP público dinâmico que muda sem aviso. O DDNS mantém um domínio fixo apontado para o IP atual da máquina — clientes e outros servidores só precisam lembrar do nome, nunca do endereço.

## Como funciona

![Arquitetura DDNS — o gerenciador detecta periodicamente o IP local, o addon atualiza o registro DNS pela API do provedor, clientes conectam por domínio](diagram/ddns)

Todo o pipeline roda automaticamente em segundo plano no servidor:

1. O **gerenciador DDNS** atualiza num ciclo fixo (a cada 5 minutos por padrão, ajustável pela variável de ambiente `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); salvar a configuração dispara uma atualização imediata, e nada roda enquanto o serviço está desabilitado;
2. A **detecção de IP local** lê primeiro a interface de rede escolhida, recorrendo a uma descoberta baseada em rotas, e filtra o resultado pelo ajuste `IP Family` (IPv4 / IPv6 / dual);
3. O **addon DDNS** — um script JS isolado — recebe a configuração descriptografada e o IP atual e chama a API do provedor DNS por HTTP. O addon Cloudflare embutido, por exemplo, localiza a zona pelo domínio e reescreve os registros A / AAAA para o IP atual, apagando registros obsoletos quando uma família de endereços desaparece;
4. Os **clientes** resolvem o domínio para o último IP e conectam por nome.

## Configuração

Abra a seção `DDNS` da interface Web:

1. Ligue o `Enable`. Você também pode deixar o serviço desligado enquanto preenche tudo — um serviço desabilitado salva sua configuração mas nunca roda;
2. escolha um provedor sob `Provider`; `Cloudflare DDNS` vem embutido;
3. preencha os campos do provedor (exemplo Cloudflare):
   - `Email` — o e-mail da conta Cloudflare;
   - `API Token` — um token com permissão Zone → DNS → Edit;
   - `Domain` — o domínio raiz da zona, p. ex. `example.com`;
   - `Subdomain` — o nome do registro dentro da zona, p. ex. `sub` para `sub.example.com`; deixe vazio para a raiz da zona;
4. em `Network Interface & IP`, escolha a `Interface` e a `IP Family`. `Dual Stack` exige que a interface tenha endereço IPv4 e IPv6;
5. clique em `Save`, depois `Sync Now` para verificar. `Sync Status` mostra o último IP enviado e a hora da sincronização, junto com a mensagem de erro do addon em caso de falha.

## Segurança

Credenciais do provedor (tokens de API) são criptografadas antes de armazenar (`enc-v2:`) e só descriptografadas em memória enquanto o addon roda; addons executam num runtime JS isolado cuja única capacidade é o HTTP de saída.

## Ampliando provedores

Provedores são addons: um manifesto JSON (declarando os campos de configuração que a interface Web transforma em formulário) mais um script JS expondo `run()`. O `cloudflare-ddns` embutido é a implementação de referência — provedores personalizados podem ser enviados como zip na seção `Addons`.

## Junto com outros recursos

- com o domínio resolvendo, o QR code da seção `Server` pode mudar `Connection Type` para `DDNS Domain`, para que clientes escaneiem uma conexão por domínio;
- outros servidores podem pôr `wss://your-domain:port` no seu `Relay Address` para rodar a cadeia de relay sobre o domínio (veja o capítulo Relay).
