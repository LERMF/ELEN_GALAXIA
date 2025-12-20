/**
 * MCP Proxy Gateway - Cloudflare Worker (SECURED)
 * Routes MCP tool calls to external services with authentication
 */

import { ExaAdapter } from './adapters/exa';
import { FirecrawlAdapter } from './adapters/firecrawl';
import { TavilyAdapter } from './adapters/tavily';
import { QdrantAdapter } from './adapters/qdrant';
import { QuantumSimAdapter } from './adapters/quantum';
import { OpenMeteoAdapter } from './adapters/open_meteo';
import { BrowserbaseAdapter } from './adapters/browser';
import { DeepResearchAdapter } from './adapters/deep_research';

export interface Env {
    MCP_VERSION: string;
    MCP_AUTH_TOKEN: string; // Required for authentication
    EXA_API_KEY: string;
    FIRECRAWL_API_KEY: string;
    TAVILY_API_KEY: string;
    BROWSERBASE_API_KEY: string;
    BRAVE_API_KEY: string;
    QDRANT_API_KEY: string;
    QDRANT_URL: string;
    CONTEXT7_API_KEY: string;
    OPENROUTER_API_KEY: string;
    GEMINI_API_KEY: string;
    GROQ_API_KEY: string;
}

interface JsonRpcRequest {
    jsonrpc: string;
    method: string;
    params?: Record<string, unknown>;
    id?: string | number | null;
}

interface JsonRpcResponse {
    jsonrpc: string;
    result?: unknown;
    error?: { code: number; message: string };
    id?: string | number | null;
}

// Tool definitions
const TOOLS = [
    {
        name: 'exa_search',
        description: 'Neural semantic search via Exa',
        inputSchema: { type: 'object', properties: { query: { type: 'string' } }, required: ['query'] }
    },
    {
        name: 'firecrawl_scrape',
        description: 'Scrape and extract content from a URL',
        inputSchema: { type: 'object', properties: { url: { type: 'string' } }, required: ['url'] }
    },
    {
        name: 'tavily_search',
        description: 'Web search via Tavily',
        inputSchema: { type: 'object', properties: { query: { type: 'string' } }, required: ['query'] }
    },
    {
        name: 'qdrant_search',
        description: 'Vector similarity search in Qdrant',
        inputSchema: { type: 'object', properties: { collection: { type: 'string' }, vector: { type: 'array' } }, required: ['collection', 'vector'] }
    },
    {
        name: 'quantum_sim',
        description: 'Ψ-Branching Quantum Simulation (Cloud)',
        inputSchema: { type: 'object', properties: { branches: { type: 'integer' } } }
    },
    {
        name: 'open_meteo_get',
        description: 'Get weather data (No Token)',
        inputSchema: { type: 'object', properties: { lat: { type: 'number' }, long: { type: 'number' }, city: { type: 'string' } } }
    },
    {
        name: 'browser_navigate',
        description: 'Navigate cloud browser via Browserbase',
        inputSchema: { type: 'object', properties: { url: { type: 'string' }, projectId: { type: 'string' } }, required: ['url'] }
    },
    {
        name: 'deep_research',
        description: 'SOTA Deep Research (Exa + Groq)',
        inputSchema: { type: 'object', properties: { query: { type: 'string' } }, required: ['query'] }
    }
];

// ============================================
// SECURITY MIDDLEWARE
// ============================================

/**
 * Validates Bearer token authentication
 */
function authenticateRequest(request: Request, env: Env): { valid: boolean; error?: string } {
    const authHeader = request.headers.get('Authorization');

    if (!authHeader) {
        return { valid: false, error: 'Missing Authorization header' };
    }

    if (!authHeader.startsWith('Bearer ')) {
        return { valid: false, error: 'Invalid Authorization format. Use: Bearer <token>' };
    }

    const token = authHeader.substring(7).trim();

    if (!env.MCP_AUTH_TOKEN) {
        console.error('MCP_AUTH_TOKEN secret not configured!');
        return { valid: false, error: 'Server configuration error' };
    }

    // Constant-time comparison to prevent timing attacks
    if (token.length !== env.MCP_AUTH_TOKEN.length) {
        return { valid: false, error: 'Invalid token' };
    }

    let mismatch = 0;
    for (let i = 0; i < token.length; i++) {
        mismatch |= token.charCodeAt(i) ^ env.MCP_AUTH_TOKEN.charCodeAt(i);
    }

    if (mismatch !== 0) {
        return { valid: false, error: 'Invalid token' };
    }

    return { valid: true };
}

/**
 * Rate limiting check (basic, can upgrade to Durable Objects for distributed)
 */
function getRateLimitHeaders(): Record<string, string> {
    return {
        'X-RateLimit-Limit': '60',
        'X-RateLimit-Remaining': '59', // Placeholder - implement with Durable Objects for real tracking
        'X-RateLimit-Reset': String(Math.floor(Date.now() / 1000) + 60)
    };
}

// ============================================
// SECURITY UTIL
// ============================================

function isValidUrl(string: string): boolean {
    try {
        const url = new URL(string);
        // Block non-http protocols (e.g. file:, javascript:, data:)
        if (url.protocol !== 'http:' && url.protocol !== 'https:') return false;
        // Block localhost/private IPs (Basic check - allows domains that might resolve to local, 
        // but blocks obvious direct IP usage. Real SSRF protection needs DNS resolution checks)
        const hostname = url.hostname;
        if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname.startsWith('192.168.') || hostname.startsWith('10.')) {
            return false;
        }
        return true;
    } catch (_) {
        return false;
    }
}

// ============================================
// HANDLERS
// ============================================

async function handleInitialize(env: Env): Promise<unknown> {
    return {
        protocolVersion: env.MCP_VERSION,
        capabilities: { tools: {} },
        // SECURITY: Obfuscate server info to prevent OSINT fingerprinting
        serverInfo: { name: 'antigravity-secure-node', version: '1.0.0-hardened' }
    };
}

async function handleListTools(): Promise<unknown> {
    return { tools: TOOLS };
}

async function handleCallTool(params: Record<string, unknown>, env: Env): Promise<unknown> {
    const name = params.name as string;
    const args = (params.arguments || {}) as Record<string, unknown>;

    // Input validation
    if (!name || typeof name !== 'string') {
        return { content: [{ type: 'text', text: 'Error: Invalid tool name' }] };
    }

    switch (name) {
        case 'exa_search':
            if (!args.query || typeof args.query !== 'string') {
                return { content: [{ type: 'text', text: 'Error: query is required' }] };
            }
            return ExaAdapter.search(args.query, env.EXA_API_KEY);
        case 'firecrawl_scrape':
            if (!args.url || typeof args.url !== 'string') {
                return { content: [{ type: 'text', text: 'Error: url is required' }] };
            }
            if (!isValidUrl(args.url)) {
                return { content: [{ type: 'text', text: 'Error: Invalid or restricted URL' }] };
            }
            return FirecrawlAdapter.scrape(args.url, env.FIRECRAWL_API_KEY);
        case 'tavily_search':
            if (!args.query || typeof args.query !== 'string') {
                return { content: [{ type: 'text', text: 'Error: query is required' }] };
            }
            return TavilyAdapter.search(args.query, env.TAVILY_API_KEY);
        case 'qdrant_search':
            if (!args.collection || !Array.isArray(args.vector)) {
                return { content: [{ type: 'text', text: 'Error: collection and vector are required' }] };
            }
            return QdrantAdapter.search(args.collection as string, args.vector as number[], env.QDRANT_URL, env.QDRANT_API_KEY);
        case 'quantum_sim':
            return QuantumSimAdapter.psiBranch((args.branches as number) || 3);
        case 'open_meteo_get':
            if (args.city) {
                const geo = await OpenMeteoAdapter.getGeo(args.city as string);
                if ((geo as any).error) return geo;
                return OpenMeteoAdapter.getWeather((geo as any).latitude, (geo as any).longitude);
            }
            if (args.lat && args.long) {
                return OpenMeteoAdapter.getWeather(args.lat as number, args.long as number);
            }
            return { content: [{ type: 'text', text: 'Error: Provide city OR lat/long' }] };
        case 'browser_navigate':
            if (!args.url || typeof args.url !== 'string') {
                return { content: [{ type: 'text', text: 'Error: url required' }] };
            }
            if (!isValidUrl(args.url)) {
                return { content: [{ type: 'text', text: 'Error: Invalid or restricted URL' }] };
            }
            return BrowserbaseAdapter.navigateAndExtract(args.url as string, env.BROWSERBASE_API_KEY, args.projectId as string);
        case 'deep_research':
            if (!args.query) return { content: [{ type: 'text', text: 'Error: query required' }] };
            return DeepResearchAdapter.research(args.query as string, env.EXA_API_KEY, env.GROQ_API_KEY);
        default:
            return { content: [{ type: 'text', text: 'Error: Unknown tool' }] }; // Don't leak tool names
    }
}

// ============================================
// MAIN HANDLER
// ============================================

export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        // Security Headers (no CORS for server-to-server communication)
        const securityHeaders = {
            'Content-Type': 'application/json',
            'X-Content-Type-Options': 'nosniff',
            'X-Frame-Options': 'DENY',
            'Cache-Control': 'no-store',
            ...getRateLimitHeaders()
        };

        // Block non-POST methods
        if (request.method === 'OPTIONS') {
            // Minimal CORS for preflight (restrictive)
            return new Response(null, {
                status: 204,
                headers: {
                    'Access-Control-Allow-Origin': '', // No cross-origin allowed
                    'Access-Control-Allow-Methods': 'POST',
                    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
                    'Access-Control-Max-Age': '86400'
                }
            });
        }

        if (request.method !== 'POST') {
            return new Response(JSON.stringify({ error: 'Method not allowed' }), {
                status: 405,
                headers: securityHeaders
            });
        }

        // ============================================
        // AUTHENTICATION CHECK
        // ============================================
        const auth = authenticateRequest(request, env);
        if (!auth.valid) {
            console.log(`Auth failed: ${auth.error} from ${request.headers.get('CF-Connecting-IP')}`);
            return new Response(JSON.stringify({
                jsonrpc: '2.0',
                error: { code: -32001, message: 'Unauthorized' }
            }), {
                status: 401,
                headers: securityHeaders
            });
        }

        // ============================================
        // PROCESS REQUEST
        // ============================================
        try {
            const body = await request.json() as JsonRpcRequest;

            // Validate JSON-RPC structure
            if (body.jsonrpc !== '2.0' || !body.method) {
                return new Response(JSON.stringify({
                    jsonrpc: '2.0',
                    error: { code: -32600, message: 'Invalid Request' }
                }), {
                    status: 400,
                    headers: securityHeaders
                });
            }

            let result: unknown;
            switch (body.method) {
                case 'initialize':
                    result = await handleInitialize(env);
                    break;
                case 'tools/list':
                    result = await handleListTools();
                    break;
                case 'tools/call':
                    result = await handleCallTool(body.params || {}, env);
                    break;
                default:
                    return new Response(JSON.stringify({
                        jsonrpc: '2.0',
                        error: { code: -32601, message: 'Method not found' },
                        id: body.id
                    }), {
                        status: 400,
                        headers: securityHeaders
                    });
            }

            const response: JsonRpcResponse = {
                jsonrpc: '2.0',
                result,
                id: body.id
            };

            return new Response(JSON.stringify(response), {
                headers: securityHeaders
            });
        } catch (err) {
            // Don't leak error details
            console.error('Request processing error:', err);
            return new Response(JSON.stringify({
                jsonrpc: '2.0',
                error: { code: -32700, message: 'Parse error' }
            }), {
                status: 400,
                headers: securityHeaders
            });
        }
    }
};
