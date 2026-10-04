// Speech model names/descriptions in the UI language. The catalog
// (crates/aura-asr/models.toml) ships Portuguese text; unknown ids fall back to it.
import type { ModelEntry } from "../ipc/types";
import type { MessageKey } from "./pt-BR";

type Translate = (key: MessageKey, vars?: Record<string, string | number>) => string;

const DESCRIPTIONS: Record<string, MessageKey> = {
  "parakeet-tdt-0.6b-v3": "voice.model.parakeetV3.description",
  "parakeet-tdt-0.6b-v2": "voice.model.parakeetV2.description",
  "whisper-base": "voice.model.whisperBase.description",
  "whisper-small": "voice.model.whisperSmall.description",
  "whisper-medium-q5": "voice.model.whisperMediumQ5.description",
  "whisper-turbo-q5": "voice.model.whisperTurboQ5.description",
  "whisper-turbo": "voice.model.whisperTurbo.description",
};

const NAMES: Record<string, MessageKey> = {
  "parakeet-tdt-0.6b-v2": "voice.model.parakeetV2.name",
};

export function modelName(t: Translate, entry: ModelEntry): string {
  const key = NAMES[entry.id];
  return key ? t(key) : entry.name;
}

export function modelDescription(t: Translate, entry: ModelEntry): string {
  const key = DESCRIPTIONS[entry.id];
  return key ? t(key) : entry.description;
}
