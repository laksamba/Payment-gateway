import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "react-toastify";
import { useAuth } from "../hooks/useAuth";
import { merchantsApi, MerchantProfile } from "../api/merchants";
import { KeyRound, Webhook, Wallet, RefreshCw, Eye, EyeOff } from "lucide-react";
import { CopyButton } from "../components/CopyButton";

function SectionCard({
  title,
  icon: Icon,
  children,
}: {
  title: string;
  icon: React.ElementType;
  children: React.ReactNode;
}) {
  return (
    <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
      <div className="px-5 py-4 border-b border-gray-200 bg-gray-50 flex items-center gap-3">
        <Icon className="w-5 h-5 text-gray-500" />
        <h3 className="font-medium text-gray-900">{title}</h3>
      </div>
      <div className="p-5">{children}</div>
    </div>
  );
}

function ApiKeySection() {
  const { apiKey } = useAuth();
  const queryClient = useQueryClient();
  const [showKey, setShowKey] = useState(false);

  const mutation = useMutation({
    mutationFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return merchantsApi(apiKey).rotateApiKey();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["profile"] });
      toast.success("API key rotated successfully");
    },
    onError: () => toast.error("Failed to rotate API key"),
  });

  return (
    <SectionCard title="API Key" icon={KeyRound}>
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-500 mb-1.5">Current API Key</label>
          <div className="flex items-center gap-2">
            <input
              type="text"
              readOnly
              value={showKey ? (apiKey ?? "") : "•".repeat(Math.min((apiKey ?? "").length, 20))}
              className="flex-1 px-4 py-2.5 border border-gray-300 rounded-lg bg-gray-50 text-sm font-mono"
            />
            <button
              onClick={() => setShowKey((s) => !s)}
              className="p-2.5 border border-gray-300 rounded-lg hover:bg-gray-50 text-gray-500"
            >
              {showKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
            {apiKey && <CopyButton text={apiKey} />}
          </div>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending}
            className="inline-flex items-center gap-2 px-4 py-2 bg-gray-900 text-white rounded-lg text-sm hover:bg-gray-800 disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${mutation.isPending ? "animate-spin" : ""}`} />
            Rotate Key
          </button>
          <p className="text-xs text-gray-500 self-center">
            Rotate if your key may be compromised. The old key becomes invalid immediately.
          </p>
        </div>
      </div>
    </SectionCard>
  );
}

function WebhookSection() {
  const { apiKey } = useAuth();
  const queryClient = useQueryClient();
  const [webhookUrl, setWebhookUrl] = useState("");

  const query = useQuery({
    queryKey: ["profile"],
    queryFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return merchantsApi(apiKey).getProfile();
    },
    enabled: !!apiKey,
  });

  const updateMutation = useMutation({
    mutationFn: (url: string) => {
      if (!apiKey) throw new Error("Not authenticated");
      return merchantsApi(apiKey).updateWebhook({ webhook_url: url });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["profile"] });
      toast.success("Webhook URL updated");
    },
    onError: () => toast.error("Failed to update webhook"),
  });

  const clearMutation = useMutation({
    mutationFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return merchantsApi(apiKey).deleteWebhook();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["profile"] });
      setWebhookUrl("");
      toast.success("Webhook cleared");
    },
    onError: () => toast.error("Failed to clear webhook"),
  });

  const rotateMutation = useMutation({
    mutationFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return merchantsApi(apiKey).rotateWebhookSecret();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["profile"] });
      toast.success("Webhook secret rotated");
    },
    onError: () => toast.error("Failed to rotate webhook secret"),
  });

  const profile: MerchantProfile | undefined = query.data;

  return (
    <SectionCard title="Webhook Settings" icon={Webhook}>
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-500 mb-1.5">Current Webhook URL</label>
          <p className="text-sm text-gray-900 bg-gray-50 px-4 py-2.5 rounded-lg border border-gray-200">
            {profile?.webhook_url || <span className="text-gray-400">Not configured</span>}
          </p>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-500 mb-1.5">Update Webhook URL</label>
          <div className="flex gap-2">
            <input
              type="url"
              value={webhookUrl}
              onChange={(e) => setWebhookUrl(e.target.value)}
              placeholder="https://yourapp.com/webhooks"
              className="flex-1 px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-green-500 outline-none text-sm"
            />
            <button
              onClick={() => updateMutation.mutate(webhookUrl)}
              disabled={!webhookUrl || updateMutation.isPending}
              className="px-4 py-2.5 bg-gray-900 text-white rounded-lg text-sm hover:bg-gray-800 disabled:opacity-50"
            >
              {updateMutation.isPending ? "Saving..." : "Save"}
            </button>
            <button
              onClick={() => clearMutation.mutate()}
              disabled={!profile?.webhook_url || clearMutation.isPending}
              className="px-4 py-2.5 border border-gray-300 text-gray-700 rounded-lg text-sm hover:bg-gray-50 disabled:opacity-50"
            >
              Clear
            </button>
          </div>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => rotateMutation.mutate()}
            disabled={rotateMutation.isPending}
            className="inline-flex items-center gap-2 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg text-sm hover:bg-gray-50 disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${rotateMutation.isPending ? "animate-spin" : ""}`} />
            Rotate Secret
          </button>
          <p className="text-xs text-gray-500 self-center">
            Rotate the webhook signing secret. Update your endpoint to use the new secret.
          </p>
        </div>
      </div>
    </SectionCard>
  );
}

function WithdrawalSection() {
  const { apiKey } = useAuth();
  const queryClient = useQueryClient();
  const [address, setAddress] = useState("");

  const mutation = useMutation({
    mutationFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return merchantsApi(apiKey).updateSettings({ withdrawal_address: address });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["profile"] });
      toast.success("Withdrawal address saved");
    },
    onError: (err: any) => {
      toast.error(err?.message ?? "Failed to save address");
    },
  });

  return (
    <SectionCard title="Withdrawal Address" icon={Wallet}>
      <div className="space-y-4">
        <p className="text-sm text-gray-500">
          Your TRON wallet address (starts with T) where USDT deposits will be swept.
        </p>
        <div className="flex gap-2">
          <input
            type="text"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="TAbc123..."
            className="flex-1 px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-green-500 outline-none text-sm font-mono"
          />
          <button
            onClick={() => mutation.mutate()}
            disabled={!address || mutation.isPending}
            className="px-4 py-2.5 bg-green-600 text-white rounded-lg text-sm hover:bg-green-700 disabled:opacity-50"
          >
            {mutation.isPending ? "Saving..." : "Save"}
          </button>
        </div>
        {address && !address.startsWith("T") && (
          <p className="text-sm text-red-500">Address must start with T (TRON network)</p>
        )}
      </div>
    </SectionCard>
  );
}

export function SettingsPage() {
  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <h2 className="text-xl font-semibold text-gray-900">Settings</h2>
        <p className="text-sm text-gray-500 mt-0.5">Manage your CryptoPay merchant configuration</p>
      </div>
      <ApiKeySection />
      <WebhookSection />
      <WithdrawalSection />
    </div>
  );
}
