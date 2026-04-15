import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useAuth } from "../hooks/useAuth";
import { paymentsApi, Payment, PaymentStatus } from "../api/payments";
import { Copy, Clock, CheckCircle, AlertCircle, RefreshCw } from "lucide-react";

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

function StatusBadge({ status }: { status: PaymentStatus }) {
  const { label, color, icon: Icon } = statusConfig[status];
  return (
    <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium ${color}`}>
      <Icon className="w-3.5 h-3.5" />
      {label}
    </span>
  );
}

export function PaymentsPage() {
  const { apiKey } = useAuth();
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
    </div>
  );
}
