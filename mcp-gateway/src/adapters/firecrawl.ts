/**
 * Firecrawl Adapter - Web Scraping
 */
export class FirecrawlAdapter {
    static async scrape(url: string, apiKey: string): Promise<unknown> {
        const response = await fetch('https://api.firecrawl.dev/v1/scrape', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${apiKey}`
            },
            body: JSON.stringify({
                url,
                formats: ['markdown']
            })
        });

        if (!response.ok) {
            return { content: [{ type: 'text', text: `Firecrawl error: ${response.status}` }] };
        }

        const data = await response.json();
        return { content: [{ type: 'text', text: JSON.stringify(data, null, 2) }] };
    }
}
