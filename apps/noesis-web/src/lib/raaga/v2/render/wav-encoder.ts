// Float32 PCM → 16-bit PCM WAV encoder. Pure browser-safe; no deps.
//
// Output is a standard RIFF/WAVE file readable by every audio app
// (QuickTime, Audacity, ffmpeg, etc).

export interface WavEncodeInput {
  /** Per-channel Float32Array buffers. Length = number of channels. */
  channels: readonly Float32Array[];
  /** Sample rate in Hz, e.g. 48000. */
  sampleRate: number;
}

const writeString = (view: DataView, offset: number, s: string): void => {
  for (let i = 0; i < s.length; i++) view.setUint8(offset + i, s.charCodeAt(i));
};

/** Encode interleaved 16-bit PCM samples + RIFF header. Returns ArrayBuffer. */
export const encodeWav = ({ channels, sampleRate }: WavEncodeInput): ArrayBuffer => {
  const numChannels = channels.length;
  if (numChannels === 0) throw new Error('encodeWav: at least one channel required');
  const numSamples = channels[0].length;
  for (const ch of channels) {
    if (ch.length !== numSamples) throw new Error('encodeWav: channel lengths differ');
  }

  const bytesPerSample = 2;  // 16-bit
  const blockAlign = numChannels * bytesPerSample;
  const byteRate = sampleRate * blockAlign;
  const dataSize = numSamples * blockAlign;
  const bufferSize = 44 + dataSize;
  const ab = new ArrayBuffer(bufferSize);
  const view = new DataView(ab);

  // RIFF header
  writeString(view, 0, 'RIFF');
  view.setUint32(4, 36 + dataSize, true);
  writeString(view, 8, 'WAVE');

  // fmt chunk
  writeString(view, 12, 'fmt ');
  view.setUint32(16, 16, true);              // PCM chunk size
  view.setUint16(20, 1, true);               // audio format = PCM
  view.setUint16(22, numChannels, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, byteRate, true);
  view.setUint16(32, blockAlign, true);
  view.setUint16(34, bytesPerSample * 8, true);

  // data chunk
  writeString(view, 36, 'data');
  view.setUint32(40, dataSize, true);

  // Interleaved PCM
  let offset = 44;
  for (let i = 0; i < numSamples; i++) {
    for (let c = 0; c < numChannels; c++) {
      const s = Math.max(-1, Math.min(1, channels[c][i]));
      const intVal = s < 0 ? s * 0x8000 : s * 0x7fff;
      view.setInt16(offset, intVal | 0, true);
      offset += 2;
    }
  }

  return ab;
};

export const wavBlobUrl = (input: WavEncodeInput): string => {
  const ab = encodeWav(input);
  const blob = new Blob([ab], { type: 'audio/wav' });
  return URL.createObjectURL(blob);
};
