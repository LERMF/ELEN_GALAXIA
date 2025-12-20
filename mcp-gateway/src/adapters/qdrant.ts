/**
 * Qdrant Adapter - Vector Similarity Search
 */
export class QdrantAdapter {
    static async search(collection: string, vector: number[], url: string, apiKey: string): Promise<unknown> {
        const response = await fetch(`${url}/collections/${collection}/points/search`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'api-key': apiKey
            },
            body: JSON.stringify({
                vector,
                limit: 5,
                with_payload: true
            })
        });

        if (!response.ok) {
            return { content: [{ type: 'text', text: `Qdrant error: ${response.status}` }] };
        }

        const data = await response.json();
        return { content: [{ type: 'text', text: JSON.stringify(data, null, 2) }] };
    }
}
