import { useCallback, useRef, useState } from 'react';

export interface LiveTranscriptionState {
  committed: string;
  unstable: string;
  isRecording: boolean;
  isConnecting: boolean;
  error: string | null;
  analyserNode: AnalyserNode | null;
}

export function useLiveTranscriber(): LiveTranscriptionState & {
  start: () => Promise<void>;
  stop: () => void;
  /** Replace the committed transcript — used by the one-click polish action. */
  setCommitted: (text: string) => void;
} {
  const [committed, setCommitted] = useState('');
  const [unstable, setUnstable] = useState('');
  const [isRecording, setIsRecording] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [analyserNode, setAnalyserNode] = useState<AnalyserNode | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const processorRef = useRef<ScriptProcessorNode | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const finalizedRef = useRef('');

  const start = useCallback(async () => {
    setCommitted('');
    setUnstable('');
    setError(null);
    setIsConnecting(true);
    finalizedRef.current = '';

    try {
      const audioCtx = new AudioContext({ sampleRate: 16000 });
      audioCtxRef.current = audioCtx;
      if (audioCtx.state === 'suspended') {
        await audioCtx.resume();
      }

      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(
        `${protocol}//${window.location.host}/api/voice/transcribe/live`,
      );
      ws.binaryType = 'arraybuffer';
      wsRef.current = ws;

      await new Promise<void>((resolve, reject) => {
        const onOpen = () => {
          cleanup();
          resolve();
        };
        const onError = () => {
          cleanup();
          reject(new Error('WebSocket connection failed'));
        };
        const cleanup = () => {
          ws.removeEventListener('open', onOpen);
          ws.removeEventListener('error', onError);
        };
        ws.addEventListener('open', onOpen);
        ws.addEventListener('error', onError);
      });

      ws.onmessage = (e) => {
        if (typeof e.data !== 'string') return;
        try {
          const msg = JSON.parse(e.data) as {
            type: string;
            committed?: string;
            unstable?: string;
            text?: string;
            message?: string;
          };
          if (msg.type === 'partial') {
            const partialCommitted = normalizeLiveText(msg.committed ?? '');
            const partialUnstable = normalizeLiveText(msg.unstable ?? '');
            setCommitted(mergeLiveText(finalizedRef.current, partialCommitted));
            setUnstable(
              limitUnstableDisplay(
                trimDuplicatedPrefix(finalizedRef.current + partialCommitted, partialUnstable),
              ),
            );
          } else if (msg.type === 'final') {
            finalizedRef.current = mergeLiveText(finalizedRef.current, msg.text ?? '');
            setCommitted(finalizedRef.current);
            setUnstable('');
          } else if (msg.type === 'error') {
            setError(msg.message ?? 'Unknown error');
          }
        } catch {
          // ignore malformed messages
        }
      };

      ws.onerror = () => setError('WebSocket error');
      ws.onclose = () => {
        setIsRecording(false);
        setIsConnecting(false);
      };

      if (!navigator.mediaDevices) {
        throw new Error(
          'Microphone access is blocked: navigator.mediaDevices is unavailable. ' +
          'This usually happens when the page is served over HTTP on a non-localhost address. ' +
          'Please use https:// or access via localhost.',
        );
      }

      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          channelCount: 1,
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
        },
      });
      streamRef.current = stream;

      if (audioCtx.state === 'suspended') {
        await audioCtx.resume();
      }

      const source = audioCtx.createMediaStreamSource(stream);
      const analyser = audioCtx.createAnalyser();
      analyser.fftSize = 2048;
      source.connect(analyser);
      setAnalyserNode(analyser);

      const processor = audioCtx.createScriptProcessor(1024, 1, 1);
      const gainNode = audioCtx.createGain();
      gainNode.gain.value = 0;

      const resampleRatio = audioCtx.sampleRate / 16000;

      processor.onaudioprocess = (e) => {
        const inputData = e.inputBuffer.getChannelData(0);
        let pcm: Float32Array;
        if (resampleRatio !== 1) {
          const outLen = Math.floor(inputData.length / resampleRatio);
          pcm = new Float32Array(outLen);
          for (let i = 0; i < outLen; i++) {
            const srcIdx = i * resampleRatio;
            const idx0 = Math.floor(srcIdx);
            const idx1 = Math.min(idx0 + 1, inputData.length - 1);
            const frac = srcIdx - idx0;
            pcm[i] = inputData[idx0]! * (1 - frac) + inputData[idx1]! * frac;
          }
        } else {
          pcm = inputData.slice();
        }
        const liveWs = wsRef.current;
        if (liveWs?.readyState === WebSocket.OPEN) {
          liveWs.send(pcm.buffer);
        }
      };

      source.connect(processor);
      processor.connect(gainNode);
      gainNode.connect(audioCtx.destination);
      processorRef.current = processor;
      gainNodeRef.current = gainNode;

      setIsRecording(true);
      setIsConnecting(false);
    } catch (err) {
      setIsConnecting(false);
      setError(err instanceof Error ? err.message : String(err));
      processorRef.current?.disconnect();
      gainNodeRef.current?.disconnect();
      audioCtxRef.current?.close().catch(() => {});
      streamRef.current?.getTracks().forEach((t) => t.stop());
      processorRef.current = null;
      gainNodeRef.current = null;
      audioCtxRef.current = null;
      streamRef.current = null;
      setAnalyserNode(null);
      wsRef.current?.close();
      wsRef.current = null;
    }
  }, []);

  const stop = useCallback(() => {
    processorRef.current?.disconnect();
    gainNodeRef.current?.disconnect();
    audioCtxRef.current?.close().catch(() => {});
    streamRef.current?.getTracks().forEach((t) => t.stop());
    processorRef.current = null;
    gainNodeRef.current = null;
    audioCtxRef.current = null;
    streamRef.current = null;
    setAnalyserNode(null);

    wsRef.current?.close();
    wsRef.current = null;

    setIsRecording(false);
    setIsConnecting(false);
  }, []);

  return {
    committed,
    unstable,
    isRecording,
    isConnecting,
    error,
    analyserNode,
    start,
    stop,
    setCommitted,
  };
}

const LIVE_PUNCTUATION = '，,。.!！?？、；;：:';
const MAX_UNSTABLE_DISPLAY_CHARS = 42;

function normalizeLiveText(text: string): string {
  return text
    .trim()
    .replace(/\s+/g, ' ')
    .replace(/\s+([，,。.!！?？、；;：:])/g, '$1')
    .replace(/([，,、]){2,}/g, '$1')
    .replace(/([。.!！?？]){2,}/g, '$1')
    .replace(/[，,、]+([。.!！?？])/g, '$1');
}

function mergeLiveText(base: string, next: string): string {
  const normalizedBase = normalizeLiveText(base);
  let normalizedNext = normalizeLiveText(next);
  if (!normalizedNext) return normalizedBase;
  if (!normalizedBase) return normalizedNext;

  if (endsWithPunctuation(normalizedBase)) {
    normalizedNext = stripLeadingPunctuation(normalizedNext);
  }

  const overlap = longestSuffixPrefixOverlap(normalizedBase, normalizedNext);
  const joined = normalizedBase + normalizedNext.slice(overlap);
  return normalizeLiveText(joined);
}

function trimDuplicatedPrefix(base: string, next: string): string {
  const normalizedNext = normalizeLiveText(next);
  if (!base || !normalizedNext) return normalizedNext;
  const overlap = longestSuffixPrefixOverlap(normalizeLiveText(base), normalizedNext);
  return normalizedNext.slice(overlap);
}

function limitUnstableDisplay(text: string): string {
  const chars = Array.from(normalizeLiveText(text));
  if (chars.length <= MAX_UNSTABLE_DISPLAY_CHARS) {
    return chars.join('');
  }
  return '...' + chars.slice(chars.length - MAX_UNSTABLE_DISPLAY_CHARS).join('');
}

function longestSuffixPrefixOverlap(base: string, next: string): number {
  const max = Math.min(base.length, next.length, 48);
  for (let len = max; len >= 2; len -= 1) {
    if (base.endsWith(next.slice(0, len))) return len;
  }
  return 0;
}

function endsWithPunctuation(text: string): boolean {
  return text.length > 0 && LIVE_PUNCTUATION.includes(text[text.length - 1]!);
}

function stripLeadingPunctuation(text: string): string {
  let index = 0;
  while (index < text.length && LIVE_PUNCTUATION.includes(text[index]!)) {
    index += 1;
  }
  return text.slice(index);
}
