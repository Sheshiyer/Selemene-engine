import { describe, it, expect } from 'vitest';
import { validateSourcePartition } from './source-partition.js';

describe('per-chapter source provenance', () => {
  const approved = new Set(['primary:house:1', 'primary:house:2']);
  const good = { source_ids: ['sw:hd:type', 'primary:house:1'],
    primary_source_ids: ['primary:house:1'], cf_source_ids: ['sw:hd:type'] };
  it('accepts an exact mixed partition', () => expect(validateSourcePartition(good, approved)).toEqual([]));
  it('accepts legacy CF-only receipts', () => expect(validateSourcePartition({source_ids:['sw:hd:type']}, approved)).toEqual([]));
  it('does not let another chapter authorise an omitted declaration', () => {
    expect(validateSourcePartition({...good, primary_source_ids:undefined}, approved)).toContain('primary_partition_mismatch');
  });
  it('rejects an authorised but unconsumed primary ID', () => {
    expect(validateSourcePartition({...good,primary_source_ids:['primary:house:1','primary:house:2']},approved)).toContain('primary_partition_mismatch');
  });
  it.each([[], ['sw:hd:type','primary:house:1'], ['sw:other'], ['sw:hd:type','sw:hd:type']].map(cf => ({cf})))('rejects altered CF complement $cf', ({cf}) => {
    expect(validateSourcePartition({...good,cf_source_ids:cf},approved)).toContain('cf_partition_mismatch');
  });
  it('requires actual CF evidence alongside primary sources', () => {
    expect(validateSourcePartition({source_ids:['primary:house:1'],primary_source_ids:['primary:house:1']},approved)).toContain('cf_partition_empty');
  });
  it('rejects duplicate source and primary IDs', () => {
    const result=validateSourcePartition({source_ids:[...good.source_ids,'primary:house:1'],primary_source_ids:['primary:house:1','primary:house:1']},approved);
    expect(result).toContain('source_ids_duplicate');expect(result).toContain('primary_ids_duplicate');
  });
  it('rejects unknown non-CF sources in a mixed receipt', () => {
    expect(validateSourcePartition({...good,source_ids:[...good.source_ids,'unknown']},approved)).toContain('cf_partition_non_cf_id');
  });
});
