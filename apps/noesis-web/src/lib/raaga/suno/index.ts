// Suno integration barrel.
export type { SunoStyle, ClipStatus, RagaClipRow, SunoCustomGenerateRequest, SunoSongResponse, BulkGenCheckpoint } from './types';
export { buildSunoPrompt, buildAllPrompts } from './prompt';
export { getQuota, submitGeneration, getSongs, pollUntilReady, downloadAudio } from './client';
export type { QuotaInfo } from './client';
export { r2KeyFor, cdnUrlFor, uploadMp3ToR2 } from './r2';
export { uploadMp3ToR2ViaWrangler } from './r2-wrangler';

// Phase 3: server-side DB helper. Don't re-export from client code —
// callers in API routes/server components should `import { findApprovedClip }
// from '@/lib/raaga/suno/db'` directly.
export type { RagaClipDbRow } from './db';
