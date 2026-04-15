import { createApiClient, ApiError } from "./client";

export interface RegisterMerchantRequest {
  name: string;
  email: string;
  password: string;
  webhook_url?: string;
}

export interface RegisterMerchantResponse {
  merchant_id: string;
  api_key: string;
  webhook_secret: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  api_key: string;
}

export interface MerchantProfile {
  merchant_id: string;
  name: string;
  email: string;
  withdrawal_address: string | null;
  webhook_url: string | null;
  fee_percent: string;
  is_active: boolean;
  created_at: string;
}

export interface UpdateWebhookRequest {
  webhook_url: string;
}

export interface RotateApiKeyResponse {
  api_key: string;
}

export interface RotateWebhookSecretResponse {
  webhook_secret: string;
}

export interface UpdateSettingsRequest {
  withdrawal_address: string;
}

export interface SavedResponse {
  saved: boolean;
}

export function merchantsApi(apiKey: string) {
  const client = createApiClient(apiKey);

  return {
    getProfile: () => client.get<MerchantProfile>("/v1/merchants/me"),

    updateWebhook: (body: UpdateWebhookRequest) =>
      client.put<SavedResponse>("/v1/merchants/me/webhook", body),

    deleteWebhook: () => client.delete<void>("/v1/merchants/me/webhook"),

    rotateApiKey: () =>
      client.post<RotateApiKeyResponse>("/v1/merchants/me/rotate-api-key", undefined),

    rotateWebhookSecret: () =>
      client.post<RotateWebhookSecretResponse>(
        "/v1/merchants/me/rotate-webhook-secret",
        undefined
      ),

    updateSettings: (body: UpdateSettingsRequest) =>
      client.post<SavedResponse>("/v1/merchants/settings", body),
  };
}

export async function registerMerchant(
  body: RegisterMerchantRequest
): Promise<RegisterMerchantResponse> {
  const res = await fetch("/v1/merchants/register", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, err.error ?? res.statusText);
  }
  return res.json();
}

export async function loginMerchant(
  body: LoginRequest
): Promise<LoginResponse> {
  const res = await fetch("/v1/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, err.error ?? res.statusText);
  }
  return res.json();
}

export { ApiError };
