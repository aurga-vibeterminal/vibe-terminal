O certificado folha só é válido para os nomes listados em seus **Nomes Alternativos de Assunto (SAN)**. Quando um cliente se conecta, o nome de host ou IP na URL dele deve corresponder exatamente a uma dessas entradas — não há suporte a curingas nem a correspondência por prefixo. Esta página cobre o gerenciamento dessa lista no servidor; a verificação do lado do cliente está na última página, Verificação estrita em clientes.

## Descoberta automática

O servidor descobre continuamente pelos quais nomes é alcançável — os nomes de host atuais e os endereços das interfaces de rede ativas — e os lista em `Descobertos automaticamente`. Essas entradas acompanham a máquina e não podem ser editadas à mão: adicione uma interface de rede ou mude o nome de host e a lista acompanhará na próxima renovação.

## Domínios e endereços IP adicionais

A descoberta não sabe de tudo: nomes DDNS, entradas de DNS de horizonte dividido ou um endereço atrás de NAT precisam ser listados manualmente. Adicione-os em `Domínios e endereços IP adicionais`:

1. Digite um nome de host ou endereço IP (por exemplo `server.home.arpa` ou `10.0.0.8`) e pressione `Adicionar`;
2. Repita para cada nome que os clientes vão usar. Remova uma entrada com o botão da lixeira;
3. Confira a `Pré-visualização final de SAN` — a visão mesclada das entradas descobertas e manuais que o próximo certificado terá;
4. Pressione `Salvar e reemitir`. Salvar assina um novo certificado folha de 90 dias com a nova lista de SAN e o recarrega a quente — as conexões existentes continuam ativas e os novos handshakes usam o novo certificado imediatamente.

## Reemissão manual e status

O painel `Certificado folha do servidor` mostra o certificado em uso: um selo de status, o assunto, a impressão digital SHA-256, o período de validade e as entradas SAN. O selo de status significa:

- `Disponível` — um certificado folha válido está carregado e em uso;
- `Falha na atualização` — a última tentativa de reemissão falhou; o certificado anterior permanece ativo e o erro é exibido na página;
- `Não gerado` — ainda não existe CA nem certificado folha; use `Gerar CA e certificado` para criar ambos.

`Reemitir certificado folha` assina um novo certificado folha imediatamente, sem esperar um gatilho automático — útil logo após mudar o nome de host ou as interfaces, ou para incorporar edições de SAN feitas fora da interface web.

## Distribuindo a CA raiz

Os clientes que vão usar a verificação estrita precisam do certificado público da CA raiz. Obtenha-o no cartão `CA raiz privada`:

- `Baixar CA raiz` salva o arquivo PEM;
- `Copiar CA raiz` coloca o PEM na área de transferência, pronto para colar no campo de CA de um cliente.

A CA nunca muda entre renovações do certificado folha nem edições de SAN — distribuí-la uma vez basta. Antes de ativar a verificação estrita em qualquer lugar, compare a impressão digital SHA-256 do certificado mostrada nesta página com o que o cliente recebeu (veja a última página, Verificação estrita em clientes).

> O download e a área de transferência contêm apenas o certificado público da CA — nunca uma chave privada. Entregue-a somente a pessoas que você quer que possam verificar seu servidor.
