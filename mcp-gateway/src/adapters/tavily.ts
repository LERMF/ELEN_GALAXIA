/**
 * Tavily Adapter - Web Search
 */
export class TavilyAdapter {
    static async search(query: string, apiKey: string): Promise<unknown> {
        const response = await fetch('https://api.tavily.com/search', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                api_key: apiKey,
                query,
                search_depth: 'basic',
                max_results: 5
            })
        });

        if (!response.ok) {
            return { content: [{ type: 'text', text: `Tavily error: ${response.status}` }] };
        }

        const data = await response.json();
        return { content: [{ type: 'text', text: JSON.stringify(data, null, 2) }] };
    }
}
