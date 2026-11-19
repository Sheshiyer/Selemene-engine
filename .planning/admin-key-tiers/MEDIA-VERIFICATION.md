# Enterprise media generation follow-up — 3 November 2026

Scope: verify real Sigil Forge images and Raaga audio with the unchanged original
Enterprise key; preserve consent gates, owner roles and scope enforcement.

- Raaga: production produced a fresh R2 WAV and reused it on repeat. Downloaded
  bytes passed RIFF/PCM, non-silence, duration, first-note frequency and content
  hash checks. No Raaga implementation change is needed.
- Sigil: NVIDIA image requests time out/return 504. Separately, the bridge's
  manager constructors ignore the configured 90-second timeout and use 30 seconds.
- Repair: honor the configured bridge timeout, bound NVIDIA calls, and support
  explicit `SIGIL_IMAGE_PROVIDER=cloudflare` using a dedicated Workers AI Read
  token restricted to the existing 9d9d account. Keep accurate provider/MIME
  metadata and omit image seed claims when the provider does not use the seed.
- Local provider execution produced a genuine 1024x1024 JPEG with no fallback.
  Unit tests cover the REST boundary, failures, image validation and envelope.

Remaining delivery gates: deploy both candidate services to staging; verify the
  API image/audio responses and consent guards; merge the reviewed source into
  main; await main deployments; verify production through the public API with
  the original key and retain the real artifacts. Private operation receipts:
  `~/.codex/private/selemene-media-generation-2026-11-03/`.

Recovery: restore the saved service variables and previous deployment. Leave
  existing R2 objects/history intact. Revoke the dedicated AI token only after
  both environments stop using it. Do not rotate the Selemene key.
