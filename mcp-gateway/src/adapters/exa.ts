/**
 * Exa Search Adapter - Neural Semantic Search
 */
export class ExaAdapter {
    static async search(query: string, apiKey: string): Promise<unknown> {
        const response = await fetch('https://api.exa.ai/search', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'x-api-key': apiKey
            },
            body: JSON.stringify({
                query,
                numResults: 5,
                useAutoprompt: true
            })
        });

        if (!response.ok) {
            return { content: [{ type: 'text', text: `Exa error: ${response.status}` }] };
        }

        const data = await response.json();
        return { content: [{ type: 'text', text: JSON.stringify(data, null, 2) }] };
    }
}
