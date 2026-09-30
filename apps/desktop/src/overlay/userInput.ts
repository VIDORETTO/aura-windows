// Shapes of agent-initiated requests (Codex protocol, pinned version) and the
// answers they expect. Pure so the mapping is unit-tested.

import type { ApprovalDecision } from "../ipc/types";

export interface Question {
  id: string;
  header: string;
  question: string;
  isOther: boolean;
  options: { label: string; description?: string }[];
}

export interface SchemaField {
  key: string;
  title: string;
  description?: string;
  kind: "string" | "number" | "integer" | "boolean" | "enum";
  options?: string[];
  required: boolean;
  default?: unknown;
}

export type InputForm =
  | { kind: "questions"; questions: Question[] }
  | { kind: "form"; message: string; fields: SchemaField[] }
  | { kind: "permissions"; reason: string | null; permissions: unknown }
  | { kind: "unknown"; raw: unknown };

type Obj = Record<string, unknown>;
const obj = (v: unknown): Obj => (v && typeof v === "object" ? (v as Obj) : {});
const str = (v: unknown, d = ""): string => (typeof v === "string" ? v : d);

export function parseInput(source: string, prompt: unknown): InputForm {
  const p = obj(prompt);
  if (source === "permissions") {
    return { kind: "permissions", reason: typeof p.reason === "string" ? p.reason : null, permissions: p.permissions ?? p };
  }
  if (Array.isArray(p.questions)) {
    return {
      kind: "questions",
      questions: p.questions.map((q, i) => {
        const o = obj(q);
        return {
          id: str(o.id, `q${i}`),
          header: str(o.header),
          question: str(o.question),
          isOther: o.isOther === true,
          options: Array.isArray(o.options) ? o.options.map((x) => ({ label: str(obj(x).label), description: str(obj(x).description) || undefined })) : [],
        };
      }),
    };
  }
  const schema = obj(p.requestedSchema);
  if (schema.properties) {
    const required = Array.isArray(schema.required) ? (schema.required as string[]) : [];
    const fields: SchemaField[] = Object.entries(obj(schema.properties)).map(([key, v]) => {
      const f = obj(v);
      const options = Array.isArray(f.enum) ? (f.enum as unknown[]).map(String) : undefined;
      const t = str(f.type, "string");
      return {
        key,
        title: str(f.title, key),
        description: str(f.description) || undefined,
        kind: options ? "enum" : t === "boolean" || t === "number" || t === "integer" ? t : "string",
        options,
        required: required.includes(key),
        default: f.default,
      };
    });
    return { kind: "form", message: str(p.message), fields };
  }
  return { kind: "unknown", raw: prompt };
}

/** Answer for `requestUserInput`: `{answers: {id: {answers: [..]}}}`. */
export function questionsAnswer(values: Record<string, string>): ApprovalDecision {
  const answers: Record<string, { answers: string[] }> = {};
  for (const [id, v] of Object.entries(values)) if (v.trim()) answers[id] = { answers: [v.trim()] };
  return { type: "answer", content: { answers } };
}

/** Coerces raw form strings to the schema types; returns missing required keys. */
export function formAnswer(fields: SchemaField[], values: Record<string, string | boolean>): { decision: ApprovalDecision; missing: string[] } {
  const content: Record<string, unknown> = {};
  const missing: string[] = [];
  for (const f of fields) {
    const raw = values[f.key];
    if (f.kind === "boolean") {
      content[f.key] = raw === true || raw === "true";
      continue;
    }
    const s = typeof raw === "string" ? raw.trim() : "";
    if (!s) {
      if (f.required) missing.push(f.key);
      continue;
    }
    if (f.kind === "number" || f.kind === "integer") {
      const n = Number(s.replace(",", "."));
      if (Number.isNaN(n) || (f.kind === "integer" && !Number.isInteger(n))) missing.push(f.key);
      else content[f.key] = n;
    } else content[f.key] = s;
  }
  return { decision: { type: "answer", content }, missing };
}
