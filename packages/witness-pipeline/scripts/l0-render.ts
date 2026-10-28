import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderLocalArtifacts } from '../src/assets/render-pipeline.js';

export const PREMIUM_ASSETS_ROOT_ENV = 'PREMIUM_ASSETS_ROOT';

/** Root of the premium-assets factory, i.e. `.../723/witness-agents-archive`. */
export function resolvePremiumAssetsRoot(explicitRoot?: string): string {
  const root = explicitRoot ?? process.env[PREMIUM_ASSETS_ROOT_ENV];
  if (!root) {
    throw new Error(
      `No premium-assets root: pass one or set ${PREMIUM_ASSETS_ROOT_ENV} to the witness-agents-archive directory`,
    );
  }
  return root;
}

function buildL0PersonDir(personId: string, root?: string): string {
  return join(resolvePremiumAssetsRoot(root), `.premium-assets-witness-${personId}`, personId);
}

export function buildL0ArtifactPath(personId: string, root?: string): string {
  return join(buildL0PersonDir(personId, root), 'local');
}

export function buildL0SourcePackPath(personId: string, root?: string): string {
  return join(buildL0PersonDir(personId, root), 'source-pack');
}

export async function renderL0Local(personId: string, root?: string) {
  return renderLocalArtifacts({
    sourcePackDir: buildL0SourcePackPath(personId, root),
    outputDir: buildL0ArtifactPath(personId, root),
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const personId = process.argv[2];
  if (!personId) {
    console.error('Usage: PREMIUM_ASSETS_ROOT=<dir> BRAND_CONFIG_PATH=<brand-config.yaml> tsx scripts/l0-render.ts <person-id>');
    process.exit(1);
  }
  renderL0Local(personId)
    .then((rendered) => console.log('Rendered artifacts:', rendered.htmlPath, rendered.pdfPath))
    .catch((e) => {
      console.error(e);
      process.exit(1);
    });
}
