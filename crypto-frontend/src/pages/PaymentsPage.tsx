import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "react-toastify";
import { useAuth } from "../hooks/useAuth";
import { paymentsApi, CreatePaymentRequest, Payment, PaymentStatus } from "../api/payments";
import { Copy, ExternalLink, Clock, CheckCircle, AlertCircle, RefreshCw, Plus } from "lucide-react";

const statusConfig: Record<PaymentStatus, { label: string; color: string; icon: React.ElementType }> = {
  pending: { label: "Pending", color: "bg-yellow-100 text-yellow-700", icon: Clock },
  detected: { label: "Detected", color: "bg-blue-100 text-blue-700", icon: AlertCircle },
  confirming: { label: "Confirming", color: "bg-orange-100 text-orange-700", icon: RefreshCw },
  confirmed: { label: "Confirmed", color: "bg-green-100 text-green-700", icon: CheckCircle },
  expired: { label: "Expired", color: "bg-gray-100 text-gray-600", icon: Clock },
  failed: { label: "Failed", color: "bg-red-100 text-red-700", icon: AlertCircle },
};

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <button
      onClick={copy}
      className="inline-flex items-center gap-1 text-xs text-gray-500 hover:text-gray-700"
    >
      <Copy className="w-3.5 h-3.5" />
      {copied ? "Copied!" : "Copy"}
    </button>
  );
}

function TRONLink({ address }: { address: string }) {
  return (
    <a
      href={`https://tronscan.org/#/address/${address}`}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 text-xs text-green-600 hover:text-green-700"
    >
      <ExternalLink className="w-3.5 h-3.5" />
      View
    </a>
  );
}

function StatusBadge({ status }: { status: PaymentStatus }) {
  const { label, color, icon: Icon } = statusConfig[status];
  return (
    <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium ${color}`}>
      <Icon className="w-3.5 h-3.5" />
      {label}
    </span>
  );
}

function CreatePaymentModal({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const { apiKey } = useAuth();
  const queryClient = useQueryClient();
  const [form, setForm] = useState({ amount: "", idempotencyKey: "" });
  const [step, setStep] = useState<"form" | "created">("form");
  const [created, setCreated] = useState<CreatePaymentRequest | null>(null);

  const mutation = useMutation({
    mutationFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return paymentsApi(apiKey).create({
        amount: form.amount,
        currency: "USDT",
        idempotency_key: form.idempotencyKey || undefined,
      });
    },
    onSuccess: (data) => {
      setCreated({ amount: data.amount, currency: data.currency, idempotency_key: form.idempotencyKey || undefined });
      setStep("created");
      queryClient.invalidateQueries({ queryKey: ["payments"] });
      toast.success("Payment created!");
    },
    onError: (err: any) => {
      toast.error(err?.message ?? "Failed to create payment");
    },
  });

  if (!open) return null;

  const handleClose = () => {
    setStep("form");
    setForm({ amount: "", idempotencyKey: "" });
    setCreated(null);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-lg">
        {step === "form" ? (
          <>
            <div className="px-6 py-4 border-b border-gray-200">
              <h2 className="text-lg font-semibold text-gray-900">Create Payment</h2>
            </div>
            <form
              onSubmit={(e) => { e.preventDefault(); mutation.mutate(); }}
              className="p-6 space-y-4"
            >
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Amount (USDT)
                </label>
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  value={form.amount}
                  onChange={(e) => setForm({ ...form, amount: e.target.value })}
                  placeholder="100.00"
                  className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-green-500 outline-none"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Idempotency Key (optional)
                </label>
                <input
                  type="text"
                  value={form.idempotencyKey}
                  onChange={(e) => setForm({ ...form, idempotencyKey: e.target.value })}
                  placeholder="order-12345"
                  className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-green-500 outline-none"
                />
              </div>
              <div className="flex gap-3 pt-2">
                <button
                  type="button"
                  onClick={handleClose}
                  className="flex-1 py-2.5 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={mutation.isPending}
                  className="flex-1 py-2.5 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
                >
                  {mutation.isPending ? "Creating..." : "Create"}
                </button>
              </div>
            </form>
          </>
        ) : (
          <>
            <div className="px-6 py-4 border-b border-gray-200">
              <h2 className="text-lg font-semibold text-gray-900">Payment Created</h2>
            </div>
            <div className="p-6 space-y-4">
              <div className="bg-gray-50 rounded-lg p-4 space-y-3">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500">Amount</span>
                  <span className="font-semibold text-gray-900">{created?.amount} USDT</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500">Status</span>
                  <StatusBadge status="pending" />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Deposit Address
                </label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    readOnly
                    value={mutation.data?.deposit_address ?? ""}
                    className="flex-1 px-4 py-2.5 border border-gray-300 rounded-lg bg-gray-50 text-sm"
                  />
                  <CopyButton text={mutation.data?.deposit_address ?? ""} />
                </div>
                <TRONLink address={mutation.data?.deposit_address ?? ""} />
              </div>
              <div className="flex gap-3 pt-2">
                <button
                  onClick={handleClose}
                  className="flex-1 py-2.5 bg-green-600 text-white rounded-lg hover:bg-green-700"
                >
                  Done
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export function PaymentsPage() {
  const { apiKey } = useAuth();
  const [showCreate, setShowCreate] = useState(false);
  const [statusFilter, setStatusFilter] = useState<PaymentStatus | "">("");
  const [page, setPage] = useState(1);

  const query = useQuery({
    queryKey: ["payments", statusFilter, page],
    queryFn: () => {
      if (!apiKey) throw new Error("Not authenticated");
      return paymentsApi(apiKey).list({
        status: statusFilter || undefined,
        limit: 20,
        page,
      });
    },
    enabled: !!apiKey,
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-gray-900">Payments</h2>
          <p className="text-sm text-gray-500 mt-0.5">
            {query.data
              ? `${query.data.pagination.total} total payments`
              : "Loading..."}
          </p>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="inline-flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
        >
          <Plus className="w-4 h-4" />
          New Payment
        </button>
      </div>

      <div className="flex gap-2">
        {["", "pending", "confirming", "confirmed", "expired"].map((s) => (
          <button
            key={s}
            onClick={() => { setStatusFilter(s as PaymentStatus | ""); setPage(1); }}
            className={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
              statusFilter === s
                ? "bg-green-600 text-white"
                : "bg-white border border-gray-300 text-gray-600 hover:bg-gray-50"
            }`}
          >
            {s || "All"}
          </button>
        ))}
      </div>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-gray-50 border-b border-gray-200">
              <th className="text-left font-medium text-gray-500 px-4 py-3">ID</th>
              <th className="text-left font-medium text-gray-500 px-4 py-3">Amount</th>
              <th className="text-left font-medium text-gray-500 px-4 py-3">Status</th>
              <th className="text-left font-medium text-gray-500 px-4 py-3">Address</th>
              <th className="text-left font-medium text-gray-500 px-4 py-3">Tx Hash</th>
              <th className="text-left font-medium text-gray-500 px-4 py-3">Created</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {query.isLoading ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-gray-500">
                  Loading payments...
                </td>
              </tr>
            ) : query.data?.data.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-gray-500">
                  No payments found. Create your first payment!
                </td>
              </tr>
            ) : (
              query.data?.data.map((p: Payment) => (
                <tr key={p.payment_id} className="hover:bg-gray-50">
                  <td className="px-4 py-3 font-mono text-xs text-gray-600 max-w-[120px] truncate">
                    {p.payment_id}
                  </td>
                  <td className="px-4 py-3 font-medium text-gray-900">
                    {p.amount} {p.currency}
                  </td>
                  <td className="px-4 py-3">
                    <StatusBadge status={p.status} />
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-xs text-gray-600 max-w-[80px] truncate">
                        {p.deposit_address}
                      </span>
                      <CopyButton text={p.deposit_address} />
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    {p.tx_hash ? (
                      <span className="font-mono text-xs text-gray-600 max-w-[80px] truncate">
                        {p.tx_hash}
                      </span>
                    ) : (
                      <span className="text-gray-400">—</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-gray-500">
                    {new Date(p.created_at).toLocaleDateString()}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {query.data && query.data.pagination.pages > 1 && (
        <div className="flex gap-2 justify-center">
          <button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm disabled:opacity-50 hover:bg-gray-50"
          >
            Previous
          </button>
          <span className="px-3 py-1.5 text-sm text-gray-600">
            Page {page} of {query.data.pagination.pages}
          </span>
          <button
            onClick={() => setPage((p) => p + 1)}
            disabled={page >= query.data.pagination.pages}
            className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm disabled:opacity-50 hover:bg-gray-50"
          >
            Next
          </button>
        </div>
      )}

      <CreatePaymentModal open={showCreate} onClose={() => setShowCreate(false)} />
    </div>
  );
}
