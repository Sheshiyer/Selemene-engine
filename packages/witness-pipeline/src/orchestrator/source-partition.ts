/** Bind each chapter's provenance partition independently of other chapters. */
export function validateSourcePartition(
  receipt: { source_ids: string[]; primary_source_ids?: string[]; cf_source_ids?: string[] },
  authorisedPrimaryIds: ReadonlySet<string>,
): string[] {
  const errors: string[] = [];
  const sources = receipt.source_ids;
  const primary = receipt.primary_source_ids ?? [];
  const expectedPrimary = sources.filter(id => authorisedPrimaryIds.has(id));
  const mixed = expectedPrimary.length > 0 || primary.length > 0;
  // Historical CF-only receipts have no partition fields.
  if (!mixed && receipt.cf_source_ids === undefined) return errors;
  const unique = (ids: string[]) => new Set(ids).size === ids.length;
  if (!unique(sources)) errors.push('source_ids_duplicate');
  if (!unique(primary)) errors.push('primary_ids_duplicate');
  if (JSON.stringify(primary) !== JSON.stringify(expectedPrimary)) errors.push('primary_partition_mismatch');
  const expectedCF = sources.filter(id => !authorisedPrimaryIds.has(id));
  if (expectedCF.length === 0) errors.push('cf_partition_empty');
  if (expectedCF.some(id => !id.startsWith('sw:'))) errors.push('cf_partition_non_cf_id');
  // CF declarations are optional in old receipts; when present they must
  // describe the exact ordered complement, with no overlap or omitted IDs.
  if (receipt.cf_source_ids !== undefined &&
      JSON.stringify(receipt.cf_source_ids) !== JSON.stringify(expectedCF)) {
    errors.push('cf_partition_mismatch');
  }
  return errors;
}
