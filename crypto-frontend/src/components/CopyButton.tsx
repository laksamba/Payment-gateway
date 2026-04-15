import { useState } from "react";
import { Copy } from "lucide-react";

export function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <button
      onClick={copy}
      className="inline-flex items-center gap-1 text-xs text-gray-500 hover:text-gray-700 border border-gray-300 rounded px-2 py-1 hover:bg-gray-50"
    >
      <Copy className="w-3.5 h-3.5" />
      {copied ? "Copied!" : "Copy"}
    </button>
  );
}
