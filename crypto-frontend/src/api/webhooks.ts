import { createApiClient } from "./client";

// ─── Types ────────────────────────────────────────────────────────────────────

export type WebhookStatus = "pending" | "delivered" | "failed";

export interface WebhookDelivery {
  id: string;
  payment_id: string;
  event_type: string;
  url: string;
  status: WebhookStatus;
  attempts: number;
  max_attempts: number;
  last_attempt_at: string | null;
  next_attempt_at: string | null;
  response_status: number | null;
  created_at: string;
}

export interface WebhookListResponse {
  data: WebhookDelivery[];
  pagination: {
    limit: number;
    offset: number;
    total: number;
  };
}

export type ListWebhooksParams = {
  limit?: number;
  offset?: number;
};

// ─── API ─────────────────────────────────────────────────────────────────────

export function webhooksApi(apiKey: string) {
  const client = createApiClient(apiKey);

  const buildQuery = (params: ListWebhooksParams): string => {
    const q = new URLSearchParams();
    if (params.limit != null) q.set("limit", String(params.limit));
    if (params.offset != null) q.set("offset", String(params.offset));
    const s = q.toString();
    return s ? `?${s}` : "";
  };

  return {
    list: (params: ListWebhooksParams = {}) =>
      client.get<WebhookListResponse>(`/v1/webhooks${buildQuery(params)}`),
  };
}
