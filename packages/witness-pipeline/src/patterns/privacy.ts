const PRIVATE_PATTERNS = [
  /\b(19|20)\d{2}[-/]\d{1,2}[-/]\d{1,2}\b/,           // dates
  /\b\d{1,2}:\d{2}(:\d{2})?\b/,                       // times
  /\b-?\d{1,3}\.\d{3,}\b,\s*-?\d{1,3}\.\d{3,}\b/,    // lat,lng
  /\b(lat|latitude|lng|longitude)\s*[:=]?\s*-?\d/i,
  /\b(timezone|tz)\s*[:=]?\s*[A-Za-z/]+/i,
];

export function scrubPrivateBirthData(text: string): { scrubbed: string; hadPrivate: boolean } {
  let scrubbed = text;
  let hadPrivate = false;
  for (const re of PRIVATE_PATTERNS) {
    if (re.test(scrubbed)) hadPrivate = true;
    scrubbed = scrubbed.replace(re, '[REDACTED]');
  }
  return { scrubbed, hadPrivate };
}
