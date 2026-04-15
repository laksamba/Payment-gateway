const BASE_URL = "";

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string
  ) {
    super(message);
    this.name = "ApiError";
  }
}

type RequestOptions = {
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
};

async function request<T>(
  path: string,
  options: RequestOptions = {}
): Promise<T> {
  const { method = "GET", body, headers = {} } = options;

  console.log(`[API] ${method} ${BASE_URL}${path}`);

  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers: {
      "Content-Type": "application/json",
      ...headers,
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  console.log(`[API] Response ${res.status} for ${method} ${path}`);

  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, error.error ?? res.statusText);
  }

  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

export function createApiClient(apiKey: string) {
  const authHeaders = { "X-API-Key": apiKey };

  return {
    get: <T>(path: string) =>
      request<T>(path, { method: "GET", headers: authHeaders }),

    post: <T>(path: string, body?: unknown) =>
      request<T>(path, { method: "POST", body, headers: authHeaders }),

    put: <T>(path: string, body?: unknown) =>
      request<T>(path, { method: "PUT", body, headers: authHeaders }),

    delete: <T>(path: string) =>
      request<T>(path, { method: "DELETE", headers: authHeaders }),
  };
}

export { BASE_URL };
