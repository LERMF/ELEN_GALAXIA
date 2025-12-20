/**
 * SOTA Token-less Adapter: OpenMeteo
 * Provides weather data without API keys.
 */
export class OpenMeteoAdapter {
    static async getWeather(latitude: number, longitude: number): Promise<unknown> {
        const url = `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,wind_speed_10m&hourly=temperature_2m,relative_humidity_2m,wind_speed_10m`;

        try {
            const response = await fetch(url);
            if (!response.ok) {
                return { error: `OpenMeteo Error: ${response.statusText}` };
            }
            const data = await response.json() as any;
            return {
                summary: "Current Weather",
                current: data.current,
                units: data.current_units
            };
        } catch (error) {
            return { error: `Fetch failed: ${(error as Error).message}` };
        }
    }

    static async getGeo(city: string): Promise<unknown> {
        const url = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(city)}&count=1&language=en&format=json`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                return { error: `Geo Error: ${response.statusText}` };
            }
            const data = await response.json() as any;
            if (!data.results || data.results.length === 0) {
                return { error: "City not found" };
            }
            return data.results[0];
        } catch (error) {
            return { error: `Fetch failed: ${(error as Error).message}` };
        }
    }
}
