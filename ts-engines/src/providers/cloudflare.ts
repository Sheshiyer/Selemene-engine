/** Workers AI image adapter. Uses a dedicated account-scoped inference token.
 * REST contract: https://developers.cloudflare.com/workers-ai/models/flux-1-schnell/
 * Only the documented prompt/steps fields are sent; image seed control is not claimed.
 */
import type { GeneratedImage, ImageGenOptions, ImageProvider } from './image-provider'

export const CLOUDFLARE_IMAGE_MODEL = '@cf/black-forest-labs/flux-1-schnell'

export class CloudflareImageProvider implements ImageProvider {
  readonly name = 'cloudflare'

  isAvailable(): boolean {
    return (
      /^[a-f0-9]{32}$/i.test(Bun.env.CLOUDFLARE_AI_ACCOUNT_ID ?? '') &&
      Boolean(Bun.env.CLOUDFLARE_AI_API_TOKEN?.trim())
    )
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    if (!this.isAvailable()) throw new Error('Cloudflare image generation is not configured')
    if (opts.model && opts.model !== CLOUDFLARE_IMAGE_MODEL) {
      throw new Error('Unsupported Cloudflare image model')
    }
    if (!opts.prompt.trim() || opts.prompt.length > 2048) {
      throw new Error('Cloudflare image prompt must contain 1–2048 characters')
    }
    const response = await fetch(
      `https://api.cloudflare.com/client/v4/accounts/${Bun.env.CLOUDFLARE_AI_ACCOUNT_ID}/ai/run/${CLOUDFLARE_IMAGE_MODEL}`,
      {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${Bun.env.CLOUDFLARE_AI_API_TOKEN}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ prompt: opts.prompt, steps: 4 }),
        signal: AbortSignal.timeout(60_000),
      },
    )
    // Provider error bodies may contain account details; expose only the status.
    if (!response.ok) throw new Error(`Cloudflare image generation failed (${response.status})`)
    const data = (await response.json()) as { success?: boolean; result?: { image?: string } }
    if (data.success !== true) throw new Error('Cloudflare image generation failed')
    const encoded = data.result?.image
    const bytes = typeof encoded === 'string' ? Buffer.from(encoded, 'base64') : Buffer.alloc(0)
    const mime = bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
      ? 'image/png'
      : bytes[0] === 255 && bytes[1] === 216 && bytes[2] === 255
        ? 'image/jpeg'
        : null
    if (!encoded || !mime) throw new Error('Cloudflare returned no valid image')
    return {
      b64_json: encoded,
      metadata: {
        model: CLOUDFLARE_IMAGE_MODEL,
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        mime_type: mime,
        finish_reason: 'SUCCESS',
      },
    }
  }
}
