# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `Rust508/UI253/typecheck/fmt/Clippy/licenses-sources; production DirectML build; MSI/PE0.3 and permanent-key independent signature/tamper verification; source snapshot+Gitleaks classification`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:2b7d17cb7848807c770fe9263a6faf05dd9e9e083dd413e76ad264420eb5b4eb`
- Timestamp: `2026-10-11T04:25:37+00:00`
- Observations: Todos os checks locais passaram; 39Rust opt-in ignorados; zero novos segredos e todos os 20findings correspondem às linhas previamente auditadas. Instalação0.3, hashes e assinaturas corretos. Remotos ainda não publicados.
- Evidence refs: none
- Limitations: Push/release/download remoto/branches pendentes; DPI150/200 e Windows limpo não testados.

## EV-002 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `Local checks+source audit; push main and annotated v0.3; draft seven-asset download/hash verification; publish latest; anonymous seven downloads/feed+API; permanent-key signature/tamper verification; ancestor checks then remove remote/local old branch; final repo metadata/actions/branch/release inventory.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:a99b6db219ab06369db9a3621b28ec0cd5f0195140b4ef4455a9d06933ec5f54`
- Timestamp: `2026-10-11T04:29:47+00:00`
- Observations: Source15c11ff published; releasev0.3.0 public/non-prerelease2026-10-11T04:27:36Z;7HTTP200 assets exact SHA and both signatures verify; latest endpoint200/version0.3;onlymain remote/local;oldreleases preserved;description/homepage/topics current;Actionsfalse. No secret/private runtime profiles committed.
- Evidence refs: none
- Limitations: No fresh nativeDPI150/200 or clean-Windows installation/updater execution; previous live/synthesis/native proofs historical and explicitly revalidated for client-version metadata.
