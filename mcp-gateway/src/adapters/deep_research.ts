/**
 * SOTA Deep Research Adapter
 * Combines Exa (Neural Search) + Groq (LPU Inference) for rapid, deep answers.
 */
export class DeepResearchAdapter {
    static async research(query: string, exaKey: string, groqKey: string): Promise<unknown> {
        if (!exaKey || !groqKey) return { error: "Missing EXA or GROQ API keys" };

        // 1. Deep Search with Exa
        let searchResults: string;
        try {
            const exaResp = await fetch('https://api.exa.ai/search', {
                method: 'POST',
                headers: {
                    'x-api-key': exaKey,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    query: query,
                    numResults: 5,
                    useAutoprompt: true,
                    contents: { text: true }
                })
            });

            if (!exaResp.ok) return { error: `Exa Search Failed: ${exaResp.statusText}` };
            const data = await exaResp.json() as any;
            searchResults = JSON.stringify(data.results.map((r: any) => ({
                title: r.title,
                url: r.url,
                text: r.text.substring(0, 500) // Truncate for context window
            })));
        } catch (e) {
            return { error: `Exa Error: ${(e as Error).message}` };
        }

        // 2. Synthesize with Groq (Mixtral 8x7b or Llama3)
        try {
            const groqResp = await fetch('https://api.groq.com/openai/v1/chat/completions', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${groqKey}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    model: "llama3-70b-8192", // SOTA fast model
                    messages: [
                        {
                            role: "system",
                            content: "You are a Deep Research Assistant. Summarize the user's query based STRICTLY on the provided search results. Cite sources."
                        },
                        {
                            role: "user",
                            content: `Query: ${query}\n\nSearch Results:\n${searchResults}`
                        }
                    ],
                    temperature: 0.2
                })
            });

            if (!groqResp.ok) return { error: `Groq Inference Failed: ${groqResp.statusText}` };
            const data = await groqResp.json() as any;
            return {
                answer: data.choices[0].message.content,
                sources: JSON.parse(searchResults).map((s: any) => s.url)
            };
        } catch (e) {
            return { error: `Groq Error: ${(e as Error).message}` };
        }
    }
}
