import type { ApiResponse } from "../../../primitives/typescript/src/types.js";

import { UnifiApiError } from "./errors.js";

export interface UnifiClientOptions {
  /** Required for authenticated calls. Wallet authentication can start without a key. */
  apiKey?: string;
  baseUrl?: string;
  fetch?: typeof globalThis.fetch;
}

interface RequestOptions {
  method?: "GET" | "POST" | "DELETE";
  query?: object;
  body?: object;
  auth?: false;
}

export function pathSegment(value: string): string {
  if (value === "" || value === "." || value === "..") {
    throw new TypeError("UniFi path parameters must be nonempty and cannot be dot segments");
  }
  return encodeURIComponent(value);
}

export function createTransport({ apiKey, baseUrl = "https://api.payunifi.com", fetch }: UnifiClientOptions) {
  if (apiKey !== undefined && !apiKey.trim()) {
    throw new TypeError("A UniFi API key is required");
  }
  const url = new URL(baseUrl);
  if (!["http:", "https:"].includes(url.protocol) || url.username || url.password || url.search || url.hash) {
    throw new TypeError("Use an HTTP(S) UniFi API base URL without credentials, a query, or a fragment");
  }
  const root = `${url.origin}${url.pathname.replace(/\/+$/, "")}`;
  const endpoint = root.endsWith("/v1") ? root : `${root}/v1`;
  const fetchRequest = fetch ?? globalThis.fetch.bind(globalThis);

  async function send(
    path: string,
    { method = "GET", body, query, auth }: RequestOptions = {},
  ): Promise<Response> {
    const headers = new Headers({ Accept: "application/json" });
    if (auth !== false) {
      if (apiKey === undefined) throw new TypeError("A UniFi API key is required");
      headers.set("Authorization", `Bearer ${apiKey}`);
    }
    if (body !== undefined) headers.set("Content-Type", "application/json");

    const requestUrl = new URL(`${endpoint}${path}`);
    for (const [name, value] of Object.entries(query ?? {})) {
      if (value !== undefined) requestUrl.searchParams.set(name, String(value));
    }
    const response = await fetchRequest(requestUrl.href, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!response.ok) throw new UnifiApiError(response.status, await response.text());
    return response;
  }

  async function request<T>(path: string, options?: RequestOptions): Promise<T> {
    const response = await send(path, options);
    const result: unknown = await response.json();
    if (
      typeof result !== "object" ||
      result === null ||
      !("status" in result) ||
      typeof result.status !== "string" ||
      !("data" in result)
    ) {
      throw new Error("Invalid UniFi API response");
    }
    return (result as ApiResponse<T>).data;
  }

  async function requestVoid(path: string, options?: RequestOptions): Promise<void> {
    const response = await send(path, options);
    await response.body?.cancel();
  }

  return { request, requestVoid };
}

export type Transport = ReturnType<typeof createTransport>;
