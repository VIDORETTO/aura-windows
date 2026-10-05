---
name: aura-configurar
description: Use quando o usuário pedir para configurar, ajustar, ligar, desligar ou mudar algo no próprio Aura (tema, opacidade, idioma, voz, memórias, ocultar em transmissões, leitura em voz alta, instruções pessoais) ou disser que não sabe onde fica uma opção. Muda com as ferramentas settings_*.
---

# Configurar o Aura por conversa

O usuário descreve o que quer; você traduz em mudanças de Configurações, mostra o efeito e só então aplica.

## Passos

1. Chame `settings_describe` para ver as opções que você pode mudar e os valores atuais.
2. Entenda o pedido. Se faltar algo que muda o resultado, faça **uma** pergunta curta com a resposta recomendada; senão siga com um padrão sensato.
3. Chame `settings_propose` com `changes` (chave → valor). Se vier erro, corrija e tente de novo uma vez.
4. Explique a mudança em linguagem simples, no formato "antes → depois". Se algum item vier com `widens_exposure: true` (amplia o que o Aura captura, guarda ou mostra a outras pessoas), avise com clareza o efeito e peça confirmação explícita.
5. Depois do "sim" do usuário, chame `settings_apply` com as mesmas mudanças. O usuário ainda aprova a chamada.
6. Diga o que mudou e que dá para desfazer (`settings_undo` ou em Configurações).

## Limites

- Nunca peça nem escreva senhas, chaves ou tokens. Chaves de provedores são digitadas pelo usuário em Configurações › Provedores.
- Você não muda modo YOLO, atalhos globais, janelas excluídas nem apaga dados: diga onde o usuário faz isso (Configurações › Geral, Atalhos, Privacidade, Diagnóstico).
- Pedidos para **reduzir** exposição ("não quero que apareça na transmissão") são simples; pedidos para **ampliar** pedem confirmação.
- Uma coisa de cada vez quando o pedido for vago; não mude o que o usuário não pediu.
