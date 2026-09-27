const roomKeys = ['engine', 'part', 'explode', 'isolate'];

export function readEngineRoomState(params, catalog) {
  const engine = catalog.find(engine => engine.id === params.get('engine'));
  if (!engine) return null;
  const partId = engine.parts.some(part => part.id === params.get('part')) ? params.get('part') : null;
  const value = Number(params.get('explode'));
  return {
    engineId: engine.id,
    explosion: Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0,
    partId,
    isolated: Boolean(partId) && params.get('isolate') === '1',
  };
}

export function writeEngineRoomState(url, state) {
  const result = new URL(url);
  roomKeys.forEach(key => result.searchParams.delete(key));
  if (state) {
    result.searchParams.set('engine', state.engineId);
    result.searchParams.set('explode', state.explosion.toFixed(2));
    if (state.partId) result.searchParams.set('part', state.partId);
    if (state.partId && state.isolated) result.searchParams.set('isolate', '1');
  }
  return result;
}
