import { expect, it, mock } from 'bun:test'
import { generateImage } from './nvidia-image'

it('bounds NVIDIA image requests while preserving the existing artifact contract', async () => {
  const originalFetch = globalThis.fetch
  const originalKey = process.env.NVIDIA_API_KEY
  process.env.NVIDIA_API_KEY = 'fixture-provider-key'
  try {
    const fetcher = mock(async (_url: Parameters<typeof fetch>[0], options?: RequestInit) => {
      expect(options?.signal).toBeInstanceOf(AbortSignal)
      expect(options?.signal?.aborted).toBe(false)
      return Response.json({ artifacts: [{ base64: 'fixture-image', finishReason: 'SUCCESS' }] })
    })
    globalThis.fetch = Object.assign(fetcher, { preconnect: originalFetch.preconnect })
    expect(await generateImage({ prompt: 'Fixture sigil' })).toEqual({
      b64_json: 'fixture-image',
      finish_reason: 'SUCCESS',
    })
  } finally {
    globalThis.fetch = originalFetch
    if (originalKey === undefined) Reflect.deleteProperty(process.env, 'NVIDIA_API_KEY')
    else process.env.NVIDIA_API_KEY = originalKey
  }
})
