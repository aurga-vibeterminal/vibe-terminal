## O navegador avisa «Sua conexão não é privada» — isso é normal?

Sim. As portas encaminhadas, a interface Web e as conexões dos clientes compartilham o mesmo certificado autoassinado do servidor; a criptografia em si não está comprometida. Importe o certificado do servidor para o armazenamento de confiança do seu dispositivo para silenciar o aviso permanentemente.

## O Vibe Terminal é gratuito?

Sim — o cliente/servidor desktop e o servidor headless são gratuitos. Baixe e instale em quantas máquinas quiser.

## Para onde vão meus dados?

Para nenhum lugar onde você não os colocou. Sessões, credenciais (enc-v2:) e reconhecimento de voz ficam nas suas máquinas; as chaves de pareamento nunca viajam em texto puro.

## Posso expor um serviço encaminhado com segurança?

Sim. Os compartilhamentos integrados exigem token: os links gerados pelo cliente para um compartilhamento `Host with HTTP server` ou a página de Settings incluem um token de acesso, e uma conexão sem token válido e sem sessão válida é derrubada antes de qualquer resposta. Na primeira visita, o token é trocado por um cookie de sessão seguro exclusivo daquele serviço e some da URL — um ticket de Settings funciona exatamente uma vez e expira em um minuto, e parar ou reiniciar um compartilhamento invalida seus links e sessões anteriores. Trate o link completo como uma credencial e entregue-o apenas a pessoas de confiança. Um encaminhamento para o seu próprio serviço acrescenta o TLS do certificado do servidor, mas nenhuma autenticação — mantenha autenticação à frente dos serviços sensíveis e derrube um encaminhamento na hora pelo painel de encaminhamento de portas quando terminar.
