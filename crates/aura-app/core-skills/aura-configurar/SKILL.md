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

## Janelas excluídas e Perfis de aplicativo

- **Privacidade por app** ("não veja o banco", "ignore o KeePass"): chame `open_windows` para achar o processo e o título certos, `exclusion_list` para não repetir, e proponha a regra em linguagem simples. Prefira excluir pelo **processo** (vale para o app todo) e use `title_glob` só para sites no navegador. Depois do "sim", chame `exclusion_add`. Excluir só reduz o que o Aura vê.
- **Perfis** ("no VS Code, respostas curtas em TypeScript"): `profile_list` e, depois do "sim", `profile_save` com `process`, `instructions`, `attach_screen` e `default_mode` quando o usuário pedir.
- Remover ou desligar uma exclusão é com o usuário, em Configurações › Privacidade.

## Limites

- Nunca peça nem escreva senhas, chaves ou tokens. Chaves de provedores são digitadas pelo usuário em Configurações › Provedores.
- Você não muda modo YOLO, atalhos globais, nem apaga dados, e não remove exclusões: diga onde o usuário faz isso (Configurações › Geral, Atalhos, Privacidade, Diagnóstico).
- Pedidos para **reduzir** exposição ("não quero que apareça na transmissão") são simples; pedidos para **ampliar** pedem confirmação.
- Uma coisa de cada vez quando o pedido for vago; não mude o que o usuário não pediu.
