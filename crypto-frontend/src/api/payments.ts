import { createApiClient } from "./client";

// ─── Types ────────────────────────────────────────────────────────────────────

export type PaymentStatus =
  | "pending"
  | "detected"
  | "confirming"
  | "confirmed"
  | "expired"
  | "failed";

export interface CreatePaymentRequest {
  amount: string;
  currency?: string;
  idempotency_key?: string;
  metadata?: Record<string, unknown>;
}

export interface Payment {
  payment_id: string;
  amount: string;
  currency: string;
  deposit_address: string;
  status: PaymentStatus;
  tx_hash: string | null;
  confirmations: number;
  required_confirmations: number;
  idempotency_key: string | null;
  metadata: Record<string, unknown>;
  expires_at: string;
  confirmed_at: string | null;
  created_at: string;
}

export interface CreatePaymentResponse {
  payment_id: string;
  deposit_address: string;
  amount: string;
  currency: string;
  status: PaymentStatus;
  expires_at: string;
}

export interface Pagination {
  total: number;
  page: number;
  limit: number;
  pages: number;
}

export interface ListPaymentsResponse {
  data: Payment[];
  pagination: Pagination;
}

export type ListPaymentsParams = {
  status?: PaymentStatus;
  limit?: number;
  page?: number;
};

// ─── API ─────────────────────────────────────────────────────────────────────

export function paymentsApi(apiKey: string) {
  const client = createApiClient(apiKey);

  const buildQuery = (params: ListPaymentsParams): string => {
    const q = new URLSearchParams();
    if (params.status) q.set("status", params.status);
    if (params.limit != null) q.set("limit", String(params.limit));
    if (params.page != null) q.set("page", String(params.page));
    const s = q.toString();
    return s ? `?${s}` : "";
  };

  return {
    create: (body: CreatePaymentRequest) =>
      client.post<CreatePaymentResponse>("/v1/payments", body),

    list: (params: ListPaymentsParams = {}) =>
      client.get<ListPaymentsResponse>(`/v1/payments${buildQuery(params)}`),

    get: (paymentId: string) =>
      client.get<Payment>(`/v1/payments/${paymentId}`),
  };
}
