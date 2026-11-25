O servidor protege todos os pontos de extremidade TLS que expõe — a interface web, as conexões de clientes e as portas encaminhadas — com um certificado emitido pela sua própria **CA raiz privada**. A página `Certificados e TLS` na interface web é onde você gerencia essa CA, controla para quais nomes o certificado é válido e entrega a CA aos clientes que desejam verificar o servidor rigorosamente.

## Como funciona

- O servidor gera uma CA raiz privada uma única vez e a mantém durante toda a vida da instalação. Tudo é assinado por ela;
- A CA assina um **certificado folha** para o servidor com validade de 90 dias, cobrindo os nomes de host e endereços IP que os clientes realmente usam para alcançá-lo;
- Quando os nomes de host, as interfaces de rede ou as entradas SAN da máquina mudam, o certificado folha é reemitido automaticamente — a CA raiz nunca muda, então os clientes que já confiam nela continuam confiando no servidor;
- O novo certificado folha é trocado por meio de um resolvedor de certificados compartilhado e recarregado a quente: as conexões existentes nunca são derrubadas e os novos handshakes usam imediatamente o certificado mais recente. Se uma renovação falhar, o certificado anterior permanece ativo.

![A página Certificados e TLS: no topo o cartão de CA raiz privada, no meio o gerenciamento de SAN, embaixo os detalhes do certificado folha do servidor](images/certificates-page.jpg)

## O que você encontra na página

1. `CA raiz privada` — baixe ou copie o certificado público da CA para distribuir aos clientes (e aos seus navegadores). O download nunca contém uma chave privada;
2. `Domínios e endereços IP (SAN)` — a lista de nomes para os quais o certificado folha é válido. Descrita na próxima página, Domínios e gerenciamento de SAN;
3. `Certificado folha do servidor` — o certificado em uso: status, assunto, impressão digital SHA-256, período de validade e entradas SAN.

## Certificados TLS personalizados

Se um certificado TLS personalizado estiver configurado para o servidor, ele continua no comando: o certificado gerenciado desta página continua sendo gerado e mantido em dia, mas não substitui a identidade personalizada configurada. Enquanto uma identidade personalizada estiver ativa, a página mostra um aviso.

## Por que confiar na CA raiz

Clientes que só precisam de criptografia funcionam sem configuração. Clientes que querem *verificar* o servidor — o modo estrito descrito na última página, Verificação estrita em clientes — precisam do certificado público da CA raiz, e ele deve trafegar por um canal confiável:

> Baixe a CA raiz apenas pela interface web local deste servidor ou por um túnel confiável, e verifique a impressão digital SHA-256 (mostrada nesta página) antes de ativar a verificação estrita em qualquer lugar.

A mesma CA também assina os certificados usados pelo encaminhamento de portas (veja a página Encaminhamento de portas). Depois que um dispositivo confia na CA raiz, as URLs encaminhadas abrem sem o aviso do navegador de *a conexão não é particular*.
