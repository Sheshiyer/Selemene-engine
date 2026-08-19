import { readFile } from 'node:fs/promises';
import yaml from 'js-yaml';

export interface BrandTokens {
  colors: Record<string, string>;
  fonts: Record<string, string>;
}

const COLOR_NAME_TO_TOKEN: Record<string, string> = {
  background: 'voidBlack',
  accent: 'sacredGold',
  text: 'parchment',
  text_secondary: 'mutedSilver',
  surface: 'deepSurface',
  success: 'coherenceEmerald',
  error: 'terracotta',
  warning: 'sacredGold',
  info: 'flowIndigo',
  primary: 'witnessViolet',
  secondary: 'flowIndigo',
  signal: 'coherenceEmerald',
};

const FONT_ROLE_TO_TOKEN: Record<string, string> = {
  heading: 'display',
  header: 'display',
  body: 'body',
  mono: 'mono',
  data: 'mono',
};

function slugifyColorName(name: string): string {
  return name
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, char) => char.toUpperCase())
    .replace(/^[A-Z]/, (char) => char.toLowerCase());
}

function isHex(value: unknown): value is string {
  return typeof value === 'string' && value.startsWith('#');
}

function extractColors(doc: any): Record<string, string> {
  const colors: Record<string, string> = {};

  const brandColors = doc?.brand?.colors;
  if (brandColors && typeof brandColors === 'object') {
    for (const [key, value] of Object.entries(brandColors)) {
      const tokenName = COLOR_NAME_TO_TOKEN[key] ?? key;
      if (isHex(value)) {
        colors[tokenName] = value;
      }
    }
  }

  const palette = doc?.palette;
  if (palette && typeof palette === 'object') {
    for (const entry of Object.values(palette)) {
      if (entry && typeof entry === 'object' && 'hex' in entry && 'name' in entry) {
        const hex = entry.hex;
        const name = entry.name;
        if (isHex(hex) && typeof name === 'string') {
          const tokenName = slugifyColorName(name);
          colors[tokenName] = hex;
        }
      }
    }
  }

  return colors;
}

function extractFonts(doc: any): Record<string, string> {
  const fonts: Record<string, string> = {};

  const brandTypography = doc?.brand?.typography;
  if (brandTypography && typeof brandTypography === 'object') {
    for (const [key, value] of Object.entries(brandTypography)) {
      const tokenName = FONT_ROLE_TO_TOKEN[key] ?? key;
      if (typeof value === 'string') {
        fonts[tokenName] = value;
      }
    }
  }

  const typography = doc?.typography;
  if (typography && typeof typography === 'object') {
    for (const [key, value] of Object.entries(typography)) {
      const tokenName = FONT_ROLE_TO_TOKEN[key] ?? key;
      const fontName = value && typeof value === 'object' ? (value as { font?: unknown }).font : value;
      if (typeof fontName === 'string' && !fonts[tokenName]) {
        fonts[tokenName] = fontName;
      }
    }
  }

  return fonts;
}

export async function loadBrandTokens(configPath: string): Promise<BrandTokens> {
  const raw = await readFile(configPath, 'utf-8');
  const doc = yaml.load(raw) as any;
  return {
    colors: extractColors(doc),
    fonts: extractFonts(doc),
  };
}
