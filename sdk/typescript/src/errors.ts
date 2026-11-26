export class UnifiApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: string,
  ) {
    super(body.trim() || `UniFi API request failed (${status})`);
    this.name = "UnifiApiError";
  }
}
