# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `Host red/green (one selection chip, replace, no duplicate, removed chip only back on @seleção or after a quick command); native red on the previous release and green: WPF QA text box (UIA TextPattern) selects text, gives the foreground back to the Overlay, chip appears and is replaced on a new selection; regressions answer-shortcuts and profile-model.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 191860544E94A3AB7B22280323DAE7F9F42AC01A28AB59F58998459C66EF73DA
- Tested revision: `local:55fdb88e3698a5c1920e93e1121d6e1bbe7686e65362648baac052184405c541`
- Timestamp: `2026-10-04T19:04:00+00:00`
- Observations: Chips '❝ Aura QA seleção' then '❝ outro trecho selecionado', always one chip; previous build showed none.
- Evidence refs: none
- Limitations: Apps whose controls have no UI Automation TextPattern (e.g. WinForms text boxes) still give no selection; no synthetic Ctrl+C by design.

## EV-002 — passed

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `aura-core red/green for accentColor validation; React red/green (presets, invalid hex, custom hex, Default) and WCAG contrast unit; native red on the previous release and green choosing #e4572e in Settings, checking computed bg-accent in Settings and Overlay, after restart, and Default.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 191860544E94A3AB7B22280323DAE7F9F42AC01A28AB59F58998459C66EF73DA
- Tested revision: `local:35a2c693156ba3b87941b703a6e7e4553ba2741f4b8b4c3c91e7487a1c2d6970`
- Timestamp: `2026-10-04T19:04:00+00:00`
- Observations: bg-accent rgb(228, 87, 46) in Settings and Overlay and after restart; Default back to rgb(91, 91, 247); screenshots 012-accent-settings.png / 012-accent-overlay.png.
- Evidence refs: none
- Limitations: Tray/executable icons are fixed images and keep the default colors.
