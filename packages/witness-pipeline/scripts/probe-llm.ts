import { probeProviders } from './lib/multi-llm.js';
process.env.DEBUG_LLM = '1';
probeProviders().then(r => console.log('RESULT:', r)).catch(e => console.error('ERROR:', e.message));