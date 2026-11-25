Por padrão, um cliente criptografa cada conexão, mas não verifica quem está do outro lado — a mesma postura de confiar na primeira utilização que aceitar um certificado autoassinado em um navegador. A **verificação estrita** muda isso: o cliente verifica o certificado do servidor contra uma CA raiz que você fornece e se recusa a conectar a menos que a cadeia, a validade e as SAN passem em todas as checagens. Isso torna os ataques de intermediário contra a conexão praticamente impossíveis, ao custo de uma etapa manual: entregar ao cliente a CA raiz do servidor.

## Onde configurar

A verificação estrita é configurada por servidor no painel de edição de servidores do cliente, ao lado do endereço e da chave secreta:

1. `Certificado de CA raiz (PEM)` — um campo de várias linhas para o certificado público da CA raiz do servidor;
2. `Verificação estrita da CA (cadeia, validade e SAN)` — selecionável somente depois de informada uma CA.

![O painel de edição de servidores no cliente de desktop: campo de várias linhas de Certificado de CA raiz (PEM) com a caixa de seleção de verificação estrita abaixo](images/client-ca-fields.jpg)

## Levando a CA raiz até o cliente

Duas formas, da mesma fonte (a página `Certificados e TLS` do servidor — veja a página Domínios e gerenciamento de SAN):

- **Copiar & colar** — pressione `Copiar CA raiz` na interface web e cole o PEM no campo `Certificado de CA raiz (PEM)` do cliente;
- **Importar uma string de compartilhamento** — as strings de compartilhamento produzidas pelo cliente incorporam a CA raiz junto com o endereço e a chave. Importar tal compartilhamento preenche automaticamente o campo da CA; quem recebe só precisa marcar a caixa da verificação estrita.

A CA é a mesma para todos os clientes e nunca muda entre renovações de certificado — configurá-la uma vez por servidor basta.

## O que o modo estrito verifica

Com a caixa marcada, cada conexão com aquele servidor deve apresentar um certificado que:

1. Encadeia à CA raiz configurada — um certificado assinado por qualquer outra coisa é rejeitado;
2. Está dentro do período de validade — certificados expirados ou ainda não válidos são rejeitados;
3. Traz uma SAN correspondente ao endereço ao qual você está conectando — um certificado emitido para outros nomes é rejeitado.

Se alguma checagem falhar, a conexão é recusada com um erro que explica qual checagem falhou. Sem a caixa marcada (ou sem uma CA), as conexões continuam criptografadas, mas essas checagens são ignoradas.

## Verificar a impressão digital

Antes de confiar na CA, confirme que você tem a verdadeira: a página `Certificados e TLS` mostra a impressão digital SHA-256 da CA raiz. Compare-a com o PEM que você está prestes a colar (ou com o PEM incorporado em um compartilhamento que recebeu). Se diferirem, não ative a verificação estrita — obtenha a CA por um canal confiável em vez disso.

## Falhas comuns

- `Incompatibilidade de SAN` — você está conectando por um nome ou IP que não está no certificado. Adicione-o no servidor (veja a página Domínios e gerenciamento de SAN) e reemita;
- `Certificado expirado ou ainda não válido` — o relógio do servidor está errado ou a renovação do certificado folha falhou. Verifique o status do certificado na interface web;
- `Não assinado pela CA configurada` — a CA no campo pertence a outro servidor, ou a conexão está sendo interceptada. Compare as impressões digitais antes de prosseguir.
