const DEFAULT_URANIA_URL = "https://urania.tryambakam.space";
const READING_IFRAME_CSP =
  "default-src 'none'; base-uri 'none'; connect-src 'none'; child-src 'none'; frame-src 'none'; " +
  "form-action 'none'; object-src 'none'; media-src 'none'; " +
  "style-src 'unsafe-inline'; img-src data:; font-src data:";
const READING_IFRAME_CSP_META =
  `<meta http-equiv="Content-Security-Policy" content="${READING_IFRAME_CSP}">`;

function safeUraniaBaseUrl(configuredUrl?: string): string {
  try {
    const url = new URL(configuredUrl?.trim() || DEFAULT_URANIA_URL);
    if (url.protocol !== "https:" && url.protocol !== "http:") {
      return DEFAULT_URANIA_URL;
    }
    url.search = "";
    url.hash = "";
    return url.toString().replace(/\/$/, "");
  } catch {
    return DEFAULT_URANIA_URL;
  }
}

export function buildUraniaConversationUrl(
  stableReadingId: string,
  configuredUrl?: string
): string {
  return `${safeUraniaBaseUrl(configuredUrl)}/#/chat?reading=${encodeURIComponent(stableReadingId)}`;
}

export function wrapVerifiedHtmlForSandbox(content: string): string {
  const head = /<head(?:\s[^>]*)?>/i.exec(content);
  if (head?.index !== undefined) {
    const insertionPoint = head.index + head[0].length;
    return `${content.slice(0, insertionPoint)}${READING_IFRAME_CSP_META}${content.slice(insertionPoint)}`;
  }

  const html = /<html(?:\s[^>]*)?>/i.exec(content);
  if (html?.index !== undefined) {
    const insertionPoint = html.index + html[0].length;
    return `${content.slice(0, insertionPoint)}<head>${READING_IFRAME_CSP_META}</head>${content.slice(insertionPoint)}`;
  }

  return `<!doctype html><html><head>${READING_IFRAME_CSP_META}</head><body>${content}</body></html>`;
}
