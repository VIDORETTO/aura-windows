---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 010-distribuicao-e-qualidade
revision: 1
status: accepted
profile: standard
---

# Specification: Distribuição e qualidade

## Problem and desired result

Um app residente só é confiável se instalar sem atrito, atualizar sozinho com segurança, explicar-se no primeiro uso e provar que é leve e acessível. O resultado é a V1 pública: instalador por usuário assinado, atualização automática assinada, onboarding que leva ao primeiro turno útil em minutos, orçamentos de desempenho verificados continuamente, acessibilidade e idiomas pt-BR/en, e diagnóstico exportável sem dados sensíveis.

## Consumers and actors

- Usuário final (instala, atualiza, faz onboarding).
- Equipe do Aura (publica releases, acompanha desempenho, dá suporte).

## Scope

### Included

- Instalador NSIS por usuário (sem administrador), assinatura Authenticode, verificação do WebView2, desinstalação limpa.
- Atualização automática assinada com canais Estável/Beta e atualização da versão fixada do app-server junto com o app.
- Onboarding de primeira execução.
- Suíte de orçamento de desempenho em CI e máquina de referência.
- Acessibilidade (leitor de tela, teclado, contraste) e i18n (pt-BR, en).
- Painel de diagnóstico e exportação de logs sem dados sensíveis.

### Excluded

- Microsoft Store/MSIX (futuro), telemetria remota, ARM64 (futuro).

## User journeys and scenarios

### US-001 — Instalar e desinstalar (Priority: P1)

#### Acceptance scenarios

- **AC-001** — Dado um Windows 10 2004+/11 sem privilégios de administrador, quando o usuário executa o instalador assinado, então o Aura é instalado em `%LOCALAPPDATA%\Programs\Aura` sem pedido de UAC, cria atalho no Menu Iniciar, inicia na bandeja e o instalador tem ≤ 15 MB.
- **AC-002** — Dado um Windows sem WebView2 Runtime, quando o instalador roda, então instala o bootstrapper do WebView2 (por usuário) ou informa claramente como resolver.
- **AC-003** — Dado o Aura instalado, quando o usuário desinstala, então o programa, atalhos e autostart são removidos e o usuário escolhe se mantém ou apaga os dados (`%LOCALAPPDATA%\Aura`, credenciais no cofre, modelos).

### US-002 — Atualizar com segurança (Priority: P1)

#### Acceptance scenarios

- **AC-004** — Dado uma nova versão publicada no canal do usuário, quando o Aura verifica (na inicialização e a cada 6 h), então baixa em segundo plano, valida a assinatura e oferece "Reiniciar para atualizar" sem interromper um turno em andamento; pacote com assinatura inválida é descartado.
- **AC-005** — Dado uma atualização que muda a versão fixada do app-server, quando aplicada, então o novo app-server é baixado/verificado antes da troca e Conversas existentes continuam retomáveis.

### US-003 — Primeiro uso (Priority: P1)

#### Acceptance scenarios

- **AC-006** — Dado a primeira execução, quando o onboarding abre, então apresenta em até 5 passos: atalho (testar), "Continue with ChatGPT" ou "Usar minha chave", privacidade (modos de captura com padrões seguros e explicação), voz (modelo recomendado opcional) e um primeiro pedido guiado; pode ser pulado e retomado nas Configurações.

### US-004 — Leve de verdade (Priority: P1)

#### Acceptance scenarios

- **AC-007** — Dado a máquina de referência (Windows 11, 4 núcleos, 8 GB, SSD), quando o `aura-bench` roda o cenário ocioso (overlay oculto, capturas desligadas, app-server parado, 5 min), então o working set privado somado de `aura.exe` + processos WebView2 do Aura é ≤ 150 MB e a CPU média ≤ 0,5%.
- **AC-008** — Dado a máquina de referência, quando o `aura-bench` mede 50 aberturas a quente, então o p95 é ≤ 100 ms; e cada build de CI Windows registra as métricas e falha se regredirem > 20% em relação à linha de base.

### US-005 — Para todos (Priority: P2)

#### Acceptance scenarios

- **AC-009** — Dado o Narrador do Windows ativo, quando o usuário usa o Overlay, então todos os controles têm nome acessível, a resposta é anunciada por frase, e todo o fluxo principal (abrir, perguntar, anexar tela, aprovar, copiar) é possível só com teclado; contraste AA em temas claro, escuro e alto contraste.
- **AC-010** — Dado o idioma do Windows em inglês (ou escolha manual), quando o Aura abre, então toda a interface aparece em inglês, e a Persona responde no idioma da interface por padrão.

### US-006 — Diagnóstico e suporte (Priority: P2)

#### Acceptance scenarios

- **AC-011** — Dado a tela Diagnóstico, quando aberta, então mostra estado de: app-server (versão, estado), Gateway, servidor MCP, worker, capturas ativas, conta ChatGPT (sem tokens), espaço em disco, versões; e "Exportar diagnóstico" gera um `.zip` com logs redigidos e configuração sem segredos, sem conteúdo de conversas nem capturas.

## Requirements

- **FR-001** — O sistema MUST ser distribuído por instalador por usuário assinado, com dependências verificadas e desinstalação limpa.
- **FR-002** — O sistema MUST se atualizar automaticamente com pacotes assinados, canais e troca segura do app-server fixado.
- **FR-003** — O sistema MUST oferecer onboarding de primeira execução retomável.
- **FR-004** — O sistema MUST cumprir e monitorar os orçamentos de desempenho.
- **FR-005** — O sistema MUST ser acessível e localizado em pt-BR e en.
- **FR-006** — O sistema MUST oferecer diagnóstico local e exportação sem dados sensíveis.

## Limits, errors, and compatibility

- Certificado de assinatura (OV/EV ou Azure Trusted Signing) é pré-requisito externo; sem ele, builds internos não assinados com aviso.
- Servidor de atualização: GitHub Releases ou CDN própria; manifesto assinado com chave do Tauri updater (chave privada fora do repositório).
- Desinstalar não revoga automaticamente o acesso ao plano ChatGPT; a tela final orienta a desconectar o app em ChatGPT Settings (ou o Aura revoga se o usuário escolher apagar dados).

## Hypotheses and dependencies

- H-004: orçamento de memória atingível. Check: AC-007.
- H-001: abertura ≤ 100 ms. Check: AC-008.
- Dependências: todos os esforços anteriores para o fluxo completo; 001 TK-003 (`aura-bench`).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-011 com evidência em Windows 10 22H2 e Windows 11 (instalação limpa em VM).

### Post-delivery observation

- **SC-002** — ≥ 95% das atualizações aplicadas sem erro; taxa de desinstalação na primeira semana < 25% no beta.

## Decisions and open questions

- Canal padrão Estável; Beta opt-in nas Configurações.
- Q-005 (não bloqueia): certificado de assinatura — recomendação: Azure Trusted Signing (custo baixo, reputação SmartScreen).
