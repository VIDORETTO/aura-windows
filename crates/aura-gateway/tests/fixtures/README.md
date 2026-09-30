# Gateway fixtures

These SSE files follow the documented streaming formats of each API
(OpenAI Chat Completions `chat.completion.chunk`, DeepSeek `reasoning_content`,
Anthropic Messages events). They were **hand-written from the public
documentation**, not recorded.

Handoff task (003 TK-003..TK-006): record real streams with test keys
(`tools/record-provider-stream`) for Groq, Gemini (OpenAI-compatible),
DeepSeek, OpenAI and Anthropic, redact ids/keys, and add them next to these
files so the translator is exercised against actual provider quirks.
