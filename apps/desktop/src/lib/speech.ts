// Read answers aloud (009 AC-005/006): one answer plays at a time; a cloud
// voice asks for a one-time confirmation before any text leaves the machine.
import { create } from "zustand";
import { api, errorMessage } from "../ipc/commands";
import { useApp } from "../state/app";

interface SpeechState {
  /** Text currently playing (or loading). */
  current: string | null;
  loading: boolean;
  /** Cloud provider waiting for the user's confirmation, with the text. */
  pendingConsent: { provider: string; text: string } | null;
}

export const useSpeech = create<SpeechState>(() => ({ current: null, loading: false, pendingConsent: null }));

let audio: HTMLAudioElement | null = null;

export function stopSpeaking() {
  audio?.pause();
  audio = null;
  useSpeech.setState({ current: null, loading: false });
}

/** Speaks `text`; a second call with the same text stops it. */
export async function speakText(text: string): Promise<void> {
  if (useSpeech.getState().current === text) return stopSpeaking();
  stopSpeaking();
  useSpeech.setState({ current: text, loading: true });
  try {
    const [b64, mime] = await api.speak(text);
    if (useSpeech.getState().current !== text) return;
    const a = new Audio(`data:${mime};base64,${b64}`);
    audio = a;
    a.onended = () => {
      if (audio === a) stopSpeaking();
    };
    useSpeech.setState({ loading: false });
    try {
      await a.play();
    } catch {
      if (audio === a) stopSpeaking();
    }
  } catch (e) {
    useSpeech.setState({ current: null, loading: false });
    const err = e as { code?: string; message?: string };
    if (err?.code === "consent_required") {
      useSpeech.setState({ pendingConsent: { provider: err.message ?? "", text } });
      return;
    }
    useApp.getState().notify("warning", errorMessage(e));
  }
}

/** The user answered the cloud-voice confirmation. */
export async function answerSpeechConsent(accept: boolean): Promise<void> {
  const pending = useSpeech.getState().pendingConsent;
  useSpeech.setState({ pendingConsent: null });
  if (!accept || !pending) return;
  try {
    await api.speechConsent();
    await speakText(pending.text);
  } catch (e) {
    useApp.getState().notify("error", errorMessage(e));
  }
}
