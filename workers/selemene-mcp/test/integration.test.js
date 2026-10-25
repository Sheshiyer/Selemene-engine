import { beforeEach, afterEach, describe, it, expect, vi } from 'vitest';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';
import worker from '../src/index.ts';
import { callSelemene } from '../src/upstream.ts';

const ORIGIN='https://mcp.example.test', API='https://selemene-engine-production.up.railway.app', RESOURCE=ORIGIN+'/mcp';
const KEY_A='nk_synthetic_only_customer_A',KEY_B='nk_synthetic_only_customer_B';
const USER_A='11111111-1111-4111-8111-111111111111',USER_B='22222222-2222-4222-8222-222222222222';
const ctx={waitUntil(){}, passThroughOnException(){}};
class KV {
  data=new Map();
  async get(key,type){const entry=this.data.get(key);if(!entry||entry.expiry<Date.now()){this.data.delete(key);return null;} return (type==='json'||type?.type==='json')?JSON.parse(entry.value):entry.value;}
  async put(key,value,opts={}){this.data.set(key,{value,expiry:opts.expirationTtl?Date.now()+opts.expirationTtl*1000:Infinity});}
  async delete(key){this.data.delete(key);}
  async list({prefix=''}={}){return {keys:[...this.data.keys()].filter(k=>k.startsWith(prefix)).map(name=>({name})),list_complete:true,cursor:''};}
}
let env,calls,revoked,workflowIds;
const safeWorkflow=['tarot','i-ching','human-design','enneagram','gene-keys'];
function dispatch(path,init={}){return worker.fetch(new Request(new URL(path,ORIGIN),init),env,ctx);}
function jsonPost(path,body){return dispatch(path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});}
async function register(){
 const res=await jsonPost('/oauth/register',{client_name:'Synthetic Reviewer',redirect_uris:['https://client.example.test/callback'],grant_types:['authorization_code','refresh_token'],response_types:['code'],token_endpoint_auth_method:'none'});
 expect(res.status).toBe(201);return res.json();
}
const verifier='synthetic-pkce-verifier-long-enough-abcdefghijklmnopqrstuvwxyz0123456789';
async function authPage(client,scope='mcp:read mcp:calculate',change={}){
 const hash=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(verifier));
 const challenge=btoa(String.fromCharCode(...new Uint8Array(hash))).replaceAll('+','-').replaceAll('/','_').replaceAll('=','');
 const params=new URLSearchParams({response_type:'code',client_id:client.client_id,redirect_uri:'https://client.example.test/callback',code_challenge:challenge,code_challenge_method:'S256',resource:RESOURCE,scope,state:'synthetic-state',...change});
 for(const [k,v] of Object.entries(change))if(v===null)params.delete(k);
 return dispatch('/authorize?'+params);
}
async function consent(page,key,action='approve',extra={}){
 const html=await page.text();
 const handle=html.match(/name="handle" value="([^"]+)"/)?.[1];
 expect(handle).toBeTruthy();
 const cookie=page.headers.getSetCookie().map(c=>c.split(';')[0]).join('; ');
 return dispatch('/authorize',{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded',origin:ORIGIN,cookie,...extra},body:new URLSearchParams({handle,action,selemene_api_key:key})});
}
async function exchange(client,code,extra={}){
 return dispatch('/oauth/token',{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded'},body:new URLSearchParams({grant_type:'authorization_code',code,client_id:client.client_id,redirect_uri:'https://client.example.test/callback',code_verifier:verifier,resource:RESOURCE,...extra})});
}
async function link(key=KEY_A,scope){
 const client=await register();const page=await authPage(client,scope);expect(page.status).toBe(200);
 const approved=await consent(page,key);expect(approved.status).toBe(302);
 const code=new URL(approved.headers.get('location')).searchParams.get('code');expect(code).toBeTruthy();
 const response=await exchange(client,code);expect(response.status).toBe(200);
 return {client,...await response.json()};
}
async function mcp(token){
 const client=new Client({name:'synthetic-client',version:'1'},{capabilities:{}});
 const transport=new StreamableHTTPClientTransport(new URL(RESOURCE),{
  requestInit:{headers:{Authorization:'Bearer '+token}},
  fetch:async(input,init)=>worker.fetch(new Request(input,init),env,ctx),
 });
 await client.connect(transport);return client;
}
const birth={name:'Test Reader',date:'1990-01-01',time:'12:00',latitude:12.9716,longitude:77.5946,timezone:'Asia/Kolkata'};

beforeEach(()=>{
 env={PUBLIC_ORIGIN:ORIGIN,SELEMENE_API_ORIGIN:API,OAUTH_KV:new KV()};calls=[];revoked=new Set();workflowIds=[...safeWorkflow];
 vi.stubGlobal('fetch',vi.fn(async(input,init)=>{
  const req=new Request(input,init); const url=new URL(req.url);const key=req.headers.get('x-api-key');
  if(url.origin!==API)throw new Error('unexpected outbound host');
  calls.push({path:url.pathname,key,method:req.method,body:req.method==='POST'?await req.json():undefined});
  if(revoked.has(key)||![KEY_A,KEY_B].includes(key))return Response.json({error:'private upstream detail '+key},{status:401});
  if(url.pathname==='/api/v1/users/me')return Response.json({id:key===KEY_A?USER_A:USER_B,consciousness_level:2,tier:'free'});
  if(url.pathname==='/api/v1/engines')return Response.json({engines:['numerology','panchanga','biorhythm','human-design','biofield']});
  if(url.pathname==='/api/v1/engines/capabilities')return new Response(null,{status:404});
  if(url.pathname==='/api/v1/workflows')return Response.json({workflows:[{id:'decision-support'}]});
  if(url.pathname.endsWith('/workflows/decision-support/info'))return Response.json({engine_ids:workflowIds});
  if(url.pathname.endsWith('/execute'))return Response.json({engine_outputs:{tarot:{result:{symbol:'test'}}}});
  if(url.pathname.endsWith('/info'))return Response.json({engine_id:url.pathname.split('/')[4],required_phase:0});
  if(url.pathname.endsWith('/calculate'))return Response.json({engine_id:url.pathname.split('/')[4],result:{synthetic:true}});
  return new Response(null,{status:404});
 }));
});
afterEach(()=>{vi.unstubAllGlobals();});

describe('full OAuth + official MCP Client interoperability',()=>{
 it('advertises minimal resource scope, full authorization catalog and calculation step-up',async()=>{
  const protectedResource=await dispatch('/.well-known/oauth-protected-resource/mcp');
  expect(protectedResource.status).toBe(200);
  expect((await protectedResource.json()).scopes_supported).toEqual(['mcp:read']);
  const authorizationServer=await dispatch('/.well-known/oauth-authorization-server');
  expect(authorizationServer.status).toBe(200);
  expect((await authorizationServer.json()).scopes_supported).toEqual(['mcp:read','mcp:calculate']);
  const readConnection=await link(KEY_A,'mcp:read');
  const c=await mcp(readConnection.access_token);
  const tools=await c.listTools();
  expect(tools.tools.find(tool=>tool.name==='selemene_calculate')._meta.securitySchemes)
    .toEqual([{type:'oauth2',scopes:['mcp:read','mcp:calculate']}]);
  const result=await c.callTool({name:'selemene_calculate',arguments:{engine_id:'numerology',birth_data:birth}});
  expect(result.isError).toBe(true);
  expect(result._meta['mcp/www_authenticate'][0]).toContain('error="insufficient_scope"');
  expect(result._meta['mcp/www_authenticate'][0]).toContain('scope="mcp:read mcp:calculate"');
  expect(calls.filter(call=>call.method==='POST')).toHaveLength(0);
  await c.close();
 });

 it('links two separate customers, exact scopes, encrypted storage, discovery and isolated calculation',async()=>{
  const a=await link(KEY_A);const b=await link(KEY_B);
  expect(a.scope.split(' ')).toContain('mcp:calculate');
  const ca=await mcp(a.access_token);const cb=await mcp(b.access_token);
  const listing=await ca.listTools();expect(listing.tools).toHaveLength(7);
  const write=listing.tools.find(t=>t.name==='selemene_calculate');
  // SDK Client strips non-standard top-level fields; raw wire coverage lives in security-protocol.test.ts.
  expect(write._meta.securitySchemes).toEqual([{type:'oauth2',scopes:['mcp:read','mcp:calculate']}]);
  expect(write.annotations).toMatchObject({readOnlyHint:false,destructiveHint:false,openWorldHint:false,idempotentHint:false});
  const catalog=await ca.callTool({name:'selemene_list_engines',arguments:{}});
  expect(catalog.structuredContent.engines.find(e=>e.engine_id==='numerology').catalog_present).toBe(true);
  expect(catalog.structuredContent.capabilities_source).toBe('unknown');
  for(const c of [ca,cb]){
   const result=await c.callTool({name:'selemene_calculate',arguments:{engine_id:'numerology',birth_data:birth}});
   expect(result.isError).not.toBe(true);expect(result.structuredContent.persistence_status).toBe('not_confirmed');
  }
  expect(calls.filter(c=>c.method==='POST').map(c=>c.key)).toEqual([KEY_A,KEY_B]);
  const stored=JSON.stringify([...env.OAUTH_KV.data.values()]);expect(stored).not.toContain(KEY_A);expect(stored).not.toContain(KEY_B);
  await ca.close();await cb.close();
 });
 it('rejects missing/plain PKCE and wrong/missing resource before consent',async()=>{
  const client=await register();
  for(const change of [{code_challenge:null},{code_challenge_method:'plain'},{resource:null},{resource:'https://evil.test/mcp'}]){
   expect((await authPage(client,undefined,change)).status).toBeGreaterThanOrEqual(400);
  }
 });
 it('preserves navigation POST Origin on the consent form without allowing null Origin',async()=>{
  const client=await register();
  const page=await authPage(client);
  expect(page.headers.get('referrer-policy')).toBe('same-origin');
  expect(page.headers.get('content-security-policy')).toContain("form-action 'self'");
  expect(page.headers.get('content-security-policy')).toContain("frame-ancestors 'none'");
  const denied=await consent(page,undefined,'deny');
  expect(denied.status).toBe(302);
  expect(denied.headers.get('referrer-policy')).toBe('no-referrer');
  expect(new URL(denied.headers.get('location')).searchParams.get('error')).toBe('access_denied');
  // The remedy is the form page's policy, not accepting opaque/null origins.
  for(const action of ['approve','deny']){
   const nullOrigin=await consent(await authPage(client),KEY_A,action,{origin:'null'});
   expect(nullOrigin.status).toBe(403);
  }
  expect(calls).toHaveLength(0);
 });
 it('binds consent to browser; deny works; rejects Origin/action and chunked oversized forms',async()=>{
  const client=await register();
  for(const [action,extra] of [['approve',{origin:'https://evil.test'}],['unknown',{}],['approve',{cookie:''}]]){
   const result=await consent(await authPage(client),KEY_A,action,extra);expect(result.status).toBeGreaterThanOrEqual(400);
  }
  const deny=await consent(await authPage(client),KEY_A,'deny');expect(deny.status).toBe(302);
  expect(new URL(deny.headers.get('location')).searchParams.get('error')).toBe('access_denied');
  const large=await dispatch('/authorize',{method:'POST',headers:{origin:ORIGIN,'content-type':'application/x-www-form-urlencoded'},body:'a='+ 'x'.repeat(9000)});
  expect(large.status).toBe(413);
 });
 it('expires tokens, rejects consent/code replay, and revokes access tokens',async()=>{
  const client=await register(); const page=await authPage(client);
  const html=await page.text();const handle=html.match(/name="handle" value="([^"]+)"/)[1];
  const cookie=page.headers.getSetCookie().map(c=>c.split(';')[0]).join('; ');
  const approve=()=>dispatch('/authorize',{method:'POST',headers:{origin:ORIGIN,cookie,'content-type':'application/x-www-form-urlencoded'},body:new URLSearchParams({handle,action:'approve',selemene_api_key:KEY_A})});
  const approved=await approve();expect(approved.status).toBe(302);expect((await approve()).status).toBe(400);
  const code=new URL(approved.headers.get('location')).searchParams.get('code');
  const noResource=await dispatch('/oauth/token',{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded'},body:new URLSearchParams({grant_type:'authorization_code',code,client_id:client.client_id,redirect_uri:'https://client.example.test/callback',code_verifier:verifier})});
  expect(noResource.status).toBe(400);
  const exchangeResponse=await exchange(client,code);expect(exchangeResponse.status).toBe(200);
  const token=await exchangeResponse.json();expect((await exchange(client,code)).status).toBe(400);
  const revocation=await dispatch('/oauth/token',{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded'},body:new URLSearchParams({token:token.access_token,token_type_hint:'access_token',client_id:client.client_id})});
  expect(revocation.status).toBe(200);
  const req=()=>dispatch('/mcp',{method:'POST',headers:{authorization:'Bearer '+token.access_token,'content-type':'application/json',accept:'application/json, text/event-stream'},body:JSON.stringify({jsonrpc:'2.0',id:1,method:'tools/list',params:{}})});
  expect((await req()).status).toBe(401);
  const fresh=await link(); const now=Date.now(); const spy=vi.spyOn(Date,'now').mockReturnValue(now+3700000);
  try {
   const expired=await dispatch('/mcp',{method:'POST',headers:{authorization:'Bearer '+fresh.access_token,'content-type':'application/json',accept:'application/json, text/event-stream'},body:JSON.stringify({jsonrpc:'2.0',id:1,method:'tools/list',params:{}})});
   expect(expired.status).toBe(401);
  } finally {spy.mockRestore();}
 });
 it('denies wrong token resource, refresh downscopes, and revoked upstream key returns valid challenge',async()=>{
  const client=await register();const approved=await consent(await authPage(client),KEY_A);const code=new URL(approved.headers.get('location')).searchParams.get('code');
  expect((await exchange(client,code,{resource:'https://evil.test/mcp'})).status).toBeGreaterThanOrEqual(400);
  const token=await link();
  const refreshed=await dispatch('/oauth/token',{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded'},body:new URLSearchParams({grant_type:'refresh_token',refresh_token:token.refresh_token,client_id:token.client.client_id,scope:'mcp:read',resource:RESOURCE})});
  expect(refreshed.status).toBe(200);const narrowed=await refreshed.json();
  const c=await mcp(narrowed.access_token);
  const denied=await c.callTool({name:'selemene_calculate',arguments:{engine_id:'numerology',birth_data:birth}});expect(denied.isError).toBe(true);
  expect(calls.filter(c=>c.method==='POST')).toHaveLength(0);
  revoked.add(KEY_A);
  const result=await c.callTool({name:'selemene_list_engines',arguments:{}});
  expect(result._meta['mcp/www_authenticate'][0]).toContain(ORIGIN+'/.well-known/oauth-protected-resource/mcp');
  expect(JSON.stringify(result)).not.toContain(KEY_A);
  await c.close();
 });
});

describe('input and side-effect safety',()=>{
 it('same local preflight catches invalid calendars, zones, options and missing inputs',async()=>{
  const token=await link();const c=await mcp(token.access_token);const before=calls.length;
  for(const args of [
   {engine_id:'numerology',birth_data:{date:'1990-01-01'}},
   {engine_id:'numerology',birth_data:{...birth,date:'2026-02-30'}},
   {engine_id:'numerology',birth_data:{...birth,timezone:'Unknown/Fake'}},
   {engine_id:'numerology',birth_data:birth,parameters:{provider:'paid'}},
   {engine_id:'human-design',birth_data:{...birth,time:undefined}},
  ]) {
   const result=await c.callTool({name:'selemene_prepare_reading',arguments:args});expect(result.structuredContent.can_calculate).toBe(false);
   const blocked=await c.callTool({name:'selemene_calculate',arguments:args});expect(blocked.isError).toBe(true);
  }
  expect(calls.length).toBe(before);
  const dateOnly={...birth};delete dateOnly.time;
  const valid=await c.callTool({name:'selemene_prepare_reading',arguments:{engine_id:'numerology',birth_data:dateOnly}});
  expect(valid.structuredContent.can_calculate).toBe(true);
  await c.close();
 });
 it('workflow refuses malformed/drift metadata before POST and reports partial output',async()=>{
  const token=await link();const c=await mcp(token.access_token);const args={workflow_id:'decision-support',birth_data:birth,options:{type:5,question:'Synthetic inquiry'}};
  for(const ids of [[],['tarot'],[...safeWorkflow,'biofield'],[...safeWorkflow,null]]){
   workflowIds=ids;const result=await c.callTool({name:'selemene_run_workflow',arguments:args});expect(result.isError).toBe(true);
  }
  expect(calls.filter(c=>c.method==='POST')).toHaveLength(0);
  workflowIds=[...safeWorkflow];
  const valid=await c.callTool({name:'selemene_run_workflow',arguments:args});expect(valid.isError).not.toBe(true);
  expect(valid.structuredContent.partial).toBe(true);expect(valid.structuredContent.missing_engine_ids).toContain('gene-keys');
  await c.close();
 });
 it('rejects batch, notification calculations and UTF8 over-limit before side effects',async()=>{
  const token=await link();
  const headers={'content-type':'application/json',accept:'application/json, text/event-stream',authorization:'Bearer '+token.access_token};
  const op={jsonrpc:'2.0',method:'tools/call',params:{name:'selemene_calculate',arguments:{engine_id:'numerology',birth_data:birth}}};
  for(const body of [JSON.stringify([ {...op,id:1},{...op,id:2} ]),JSON.stringify(op)]){
   expect((await dispatch('/mcp',{method:'POST',headers,body})).status).toBe(400);
  }
  expect((await dispatch('/mcp',{method:'POST',headers,body:JSON.stringify({padding:'界'.repeat(12000)})})).status).toBe(413);
  expect(calls.filter(c=>c.method==='POST')).toHaveLength(0);
 });
 it('bounds stalled upstream body and does not retry',async()=>{
  let count=0;vi.stubGlobal('fetch',async()=>{count++;return new Response(new ReadableStream({start(){}}));});
  const result=await callSelemene({selemeneApiOrigin:API},{method:'GET',path:'/api/v1/engines',apiKey:KEY_A,timeoutMs:25});
  expect(result.kind).not.toBe('ok');expect(count).toBe(1);
 });
});
