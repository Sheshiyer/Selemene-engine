import { renderLocalArtifacts } from '../src/assets/render-pipeline.js';

export function buildL0ArtifactPath(personId: string): string {
  return `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-${personId}/${personId}/local`;
}

export async function renderL0Local(personId: string) {
  const sourcePackDir = buildL0ArtifactPath(personId).replace('/local', '/source-pack');
  const outputDir = buildL0ArtifactPath(personId);
  return renderLocalArtifacts({
    sourcePackDir,
    outputDir,
    brandConfigPath: '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/brand-config.yaml',
  });
}
