Por padrão, um servidor compartilhado criptografa sua conexão a montante com o relay, mas não verifica quem está do outro lado — a mesma postura de confiar na primeira utilização que aceitar um certificado autoassinado em um navegador. O **TLS estrito do relay** muda isso: o servidor compartilhado verifica o certificado do relay contra uma CA raiz que você fornece e se recusa a conectar a menos que a cadeia, a validade e o nome de host passem em todas as checagens. Isso torna os ataques de intermediário contra o enlace do relay praticamente impossíveis, ao custo de uma etapa manual: entregar ao servidor compartilhado a CA raiz do relay. Ele aplica a mesma ideia da página anterior, Verificação estrita em clientes, ao enlace a montante do servidor compartilhado.

## Onde configurar

A verificação estrita é configurada no servidor compartilhado — a máquina cujo `Relay Address` aponta para o relay. Na interface web dele, abra a seção `Server`: abaixo da `Relay Secret Key` fica uma linha `TLS estrito do relay` mostrando o estado `On` / `Off` atual. Expanda-a para ver:

1. `CA raiz do relay upstream` — um campo de várias linhas para o certificado público da CA raiz do relay;
2. Um seletor `Off` / `On` — `On` só fica selecionável quando o PEM é interpretado como um certificado válido.

![Interface Web — o bloco TLS estrito do relay expandido na seção Server: campo CA raiz do relay upstream com o seletor Off/On e o botão Apply](images/cert-relay-strict-tls.jpg)

As configurações do relay nunca são salvas no meio da edição: revise os valores e clique em `Apply`. Salvar reconecta imediatamente o relay a montante com as novas configurações.

## Levando a CA raiz até o servidor compartilhado

Duas formas, da mesma fonte (a interface web do próprio relay):

- **Colar uma string de compartilhamento** — clique no `QR code` na interface web do servidor relay. A string incorpora a CA raiz do relay junto com o endereço e a chave, então colá-la no campo `Relay Address` do servidor compartilhado também preenche automaticamente o campo da CA (veja o Guia de configuração em Relay). O modo estrito continua desligado até você revisar a CA e ativá-lo;
- **Copiar & colar** — pressione `Copiar CA raiz` na página `Certificados e TLS` do relay e cole o PEM no campo `CA raiz do relay upstream`.

A CA é a mesma para todos os servidores compartilhados e nunca muda entre renovações de certificado — configurá-la uma vez por relay basta.

Antes de confiar na CA, confirme que você tem a verdadeira: a página `Certificados e TLS` do relay mostra a impressão digital SHA-256 da CA raiz. Compare-a com o PEM que você está prestes a colar. Se diferirem, não ative a verificação estrita — obtenha a CA por um canal confiável em vez disso.

## O que o modo estrito verifica

Com o seletor em `On`, a conexão `RelayPrimary` a montante deve apresentar um certificado que:

1. Encadeia à CA raiz configurada — um certificado assinado por qualquer outra coisa, incluindo raízes públicas do sistema, é rejeitado;
2. Está dentro do período de validade — certificados expirados ou ainda não válidos são rejeitados;
3. Traz uma SAN correspondente ao host do `Relay Address` ao qual você está conectando — um certificado emitido para outros nomes é rejeitado.

Um limite a lembrar: apenas a conexão a montante com o relay é verificada. As conexões de retorno de Terminal e de requisições mantêm a política de certificados existente.

Se alguma checagem falhar, a conexão é recusada e repetida em segundo plano com atrasos crescentes; o servidor compartilhado fica inacessível pelo relay até a checagem passar ou você desligar o modo estrito.

## Falhas comuns

- `Verificação estrita de TLS exige um PEM de CA raiz` — o seletor está ligado mas o campo da CA está vazio. Cole o PEM primeiro;
- `Incompatibilidade de SAN` — você está chamando o relay por um nome ou IP que não está no certificado dele. Adicione-o na página `Certificados e TLS` do relay (veja a página Domínios e gerenciamento de SAN) e reemita;
- `Certificado expirado ou ainda não válido` — o relógio do relay está errado ou a renovação do certificado folha falhou. Verifique o status do certificado no relay;
- `Não assinado pela CA configurada` — a CA no campo pertence a outro relay, ou a conexão está sendo interceptada. Compare as impressões digitais antes de prosseguir.
