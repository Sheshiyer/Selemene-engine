// Suno integration types — frozen contract (Phase 1).

export type SunoStyle = 'ambient' | 'meditative' | 'cinematic' | 'acid';

export type ClipStatus = 'pending' | 'generated' | 'approved' | 'rejected' | 'regenerate';

export interface RagaClipRow {
  id: number;
  melakarta_num: number;
  style: SunoStyle;
  duration_sec: number;
  suno_song_id: string;
  suno_prompt: string;
  r2_key: string;
  cdn_url: string;
  status: ClipStatus;
  audition_notes: string | null;
  created_at: string;
  approved_at: string | null;
}

export interface SunoCustomGenerateRequest {
  prompt: string;
  tags: string;          // genre/style descriptors
  title: string;
  make_instrumental: true;  // we always want instrumental — no vocals on ragas
  wait_audio: false;     // we poll separately
  mv?: string;           // model version, optional
}

export interface SunoSongResponse {
  id: string;
  title: string;
  status: 'submitted' | 'queued' | 'streaming' | 'complete' | 'error';
  audio_url: string;     // empty until status='streaming' or 'complete'
  duration: number;      // seconds; 0 until complete
  prompt: string;
  tags: string;
  error_message?: string;
}

export interface BulkGenCheckpoint {
  /** ISO timestamp of last successful generation. */
  lastSuccessAt: string;
  /** Set of melakarta numbers already generated successfully (any style). */
  completed: Record<number, SunoStyle[]>;
  /** Total credits consumed so far. */
  creditsUsed: number;
  /** Quota at start of this run. */
  initialCredits: number;
}
