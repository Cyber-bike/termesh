import { requestUrl } from 'obsidian';

import type { RelayHttpResponse } from './relayClient';

export async function requestWithObsidian(input: URL, init: RequestInit): Promise<RelayHttpResponse> {
  const headers: Record<string, string> = {};
  new Headers(init.headers).forEach((value, key) => {
    headers[key] = value;
  });
  const response = await requestUrl({
    url: input.toString(),
    method: init.method,
    body: typeof init.body === 'string' ? init.body : undefined,
    headers,
    throw: false,
  });
  return {
    ok: response.status >= 200 && response.status < 300,
    status: response.status,
    json: () => Promise.resolve(response.json as unknown),
  };
}