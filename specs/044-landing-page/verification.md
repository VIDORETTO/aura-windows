# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `pnpm -C apps/landing test (node --test src/lib/release.test.ts)`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; Node 25.2.1
- Tested revision: `local:b31ec858b2dfbc05622c4200f6569d3fec211b9183331674e479c7545a7141f4`
- Timestamp: `2026-10-14T03:14:57+00:00`
- Observations: 5/5: instalador NSIS escolhido em vez do .sig, digest da API mantido, rascunho rejeitado, comparação de versões, tamanho em MB.
- Evidence refs: none
- Limitations: none recorded

## EV-002 — passed

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `pnpm -C apps/landing check; pnpm -C apps/landing build; pnpm -C apps/landing budget; Lighthouse 12.8.2 (Edge headless) mobile e --preset=desktop em astro preview; Playwright 1440x900 e 390x844`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; Node 25.2.1; Astro 7.3.8; Edge
- Tested revision: `local:584afbf340330998da297d80c897a7ef429fb861ef888b957f83afc2545c0958`
- Timestamp: `2026-10-14T03:14:57+00:00`
- Observations: astro check 0 erros. Budget: JS 2,1 KB, CSS 7,0 KB, HTML 16,6 KB gzip, 4 WOFF2. Lighthouse desktop 100/100/100/100 (LCP 0,6 s, CLS 0); mobile 98/100/100/100 (LCP 2,2 s, CLS 0,019). Red antes das correções: CLS 0,758 (anéis do hero) e contraste dos passos inativos. Sem erros de console; overflow horizontal 0 em 390 px.
- Evidence refs: none
- Limitations: LCP mobile acima da meta de 1,8 s; workflow do Pages não executado.
