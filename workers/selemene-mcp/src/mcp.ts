// Official SDK protocol and transport. One bounded operation per request.
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { WebStandardStreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/webStandardStreamableHttp.js";
import { CallToolRequestSchema, ListToolsRequestSchema } from "@modelcontextprotocol/sdk/types.js";
import { zodToJsonSchema } from "zod-to-json-schema";
import type { ValidatedEnv } from "./env.js";
import type { SelemeneAuthProps } from "./props.js";
import { parseScopes } from "./props.js";
import { ALL_TOOLS } from "./tools/index.js";

export interface McpRequestContext { env: ValidatedEnv; props: SelemeneAuthProps; scope: readonly string[] }
const MAX_BYTES = 32768;
const error = (status: number, message: string) => Response.json({ jsonrpc: "2.0", id: null, error: { code: -32600, message } }, {status, headers: {"cache-control":"no-store"}});
export async function handleMcp(request: Request, ctx: McpRequestContext): Promise<Response> {
  if (request.method !== "POST") return new Response(null, {status:405, headers:{Allow:"POST"}});
  if (!request.headers.get("content-type")?.startsWith("application/json")) return error(415,"Expected JSON.");
  const origin = request.headers.get("origin");
  if (origin && origin !== ctx.env.publicOrigin) return error(403,"Origin denied.");
  const chunks: Uint8Array[] = []; let size = 0; const reader = request.body?.getReader();
  try {
    if (reader) while (true) {
      const {value, done} = await reader.read(); if(done) break;
      size += value.byteLength;
      if (size > MAX_BYTES) { void reader.cancel().catch(() => {}); return error(413,"Request too large."); }
      chunks.push(value);
    }
  } catch { return error(400,"Invalid request body."); }
  const bytes = new Uint8Array(size); let offset=0;
  for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.byteLength;}
  let parsed: Record<string,unknown>;
  try { parsed=JSON.parse(new TextDecoder().decode(bytes)); } catch {return error(400,"Invalid JSON.");}
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return error(400,"One operation per request required.");
  if (parsed.method === "tools/call" && !Object.prototype.hasOwnProperty.call(parsed,"id")) return error(400,"Tool calls require a request id.");
  const scopes = parseScopes(ctx.scope);
  const server = new Server({name:"selemene-mcp",version:"0.1.0"},{capabilities:{tools:{}}});
  const schemes = (write:boolean) => [{type:"oauth2",scopes:write?["mcp:read","mcp:calculate"]:["mcp:read"]}];
  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: ALL_TOOLS.map(tool=>({
      name:tool.name,description:tool.description,
      inputSchema: zodToJsonSchema(tool.inputSchema,{$refStrategy:"none"}) as {type:"object";[key:string]:unknown},
      annotations:tool.annotations,
      securitySchemes:schemes(tool.requiredScope==="calculate"),
      _meta:{securitySchemes:schemes(tool.requiredScope==="calculate")},
    })),
  }));
  server.setRequestHandler(CallToolRequestSchema, async ({params}) => {
    const tool=ALL_TOOLS.find(item=>item.name===params.name);
    if(!tool) return {isError:true,content:[{type:"text",text:"Unknown tool."}]};
    if(!scopes.read || (tool.requiredScope==="calculate" && !scopes.calculate)) return {
      isError:true,content:[{type:"text",text:"The connection needs additional permissions."}],
      _meta:{"mcp/www_authenticate":[`Bearer resource_metadata="${ctx.env.publicOrigin}/.well-known/oauth-protected-resource/mcp", error="insufficient_scope", error_description="Reconnect and approve the required scope.", scope="mcp:read${tool.requiredScope==="calculate"?" mcp:calculate":""}"`]},
    };
    const validated=tool.inputSchema.safeParse(params.arguments??{});
    if(!validated.success) return {isError:true,content:[{type:"text",text:"Invalid tool input."}]};
    try {
      const result = await tool.handler({env:ctx.env,props:ctx.props,scope:scopes},validated.data as Record<string,unknown>);
      const secret = JSON.stringify(ctx.props.apiKey).slice(1,-1);
      return JSON.parse(JSON.stringify(result).split(secret).join("[REDACTED]"));
    }
    catch {return {isError:true,content:[{type:"text",text:"Tool execution failed."}]};}
  });
  const transport=new WebStandardStreamableHTTPServerTransport({sessionIdGenerator:undefined,enableJsonResponse:true,maxRequestBodySize:MAX_BYTES});
  try {
    await server.connect(transport);
    const forwarded=new Request(request.url,{method:"POST",headers:request.headers,body:bytes});
    const response=await transport.handleRequest(forwarded,{parsedBody:parsed});
    response.headers.set("cache-control","no-store");
    return response;
  } catch {return error(500,"Protocol operation failed.");}
  finally {await server.close().catch(()=>{});}
}
