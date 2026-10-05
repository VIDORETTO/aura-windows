# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`
- Procedure: `Service red/green against the fake app-server (turn/start params and transcript); React red/green (divider, no duplicate); native red on the previous release and green with the real pinned app-server and a loopback model: Chat start, switch to Task and Plan mid-conversation, upstream requests inspected, conversation reopened; regressions model-efforts and compact.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 548552A18E5AA23FBF17EA03DF3ADE90E4C8FF47A87A3ECF364FB2CE37395736; codex-app-server rust-v0.159.0
- Tested revision: `local:7bd268e26cf80c328ef4d001630cf78affb762f85c78d167e9cfd844964f501f`
- Timestamp: `2026-10-04T22:09:18+00:00`
- Observations: Turn 2 carried the Task mode note, turn 3 none, turn 4 the Plan mode note; start-time 'Chat mode' developer instruction still present (cause confirmed); reopened transcript shows only the user texts.
- Evidence refs: none
- Limitations: Model behaviour under the note not evaluated (loopback upstream); the sandbox policy per turn remains the enforcement.
