import { Copy, CheckCircle } from "lucide-react";
import { useState } from "react";
import { config } from "../config";

const apiBaseUrl = config.apiBaseUrl;

interface CodeBlockProps {
  title: string;
  code: string;
}

function CodeBlock({ title, code }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="bg-gray-900 rounded-lg overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800">
        <span className="text-xs text-gray-400 uppercase">{title}</span>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1 text-xs text-gray-400 hover:text-white transition-colors"
        >
          {copied ? <CheckCircle className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="p-4 overflow-x-auto">
        <code className="text-sm text-green-400 font-mono whitespace-pre">{code}</code>
      </pre>
    </div>
  );
}

function Alert({ type, children }: { type: "info" | "warning"; children: React.ReactNode }) {
  const colors = {
    info: "bg-blue-50 border-blue-200 text-blue-800",
    warning: "bg-amber-50 border-amber-200 text-amber-800",
  };
  return (
    <div className={`p-4 rounded-lg border text-sm ${colors[type]}`}>{children}</div>
  );
}

export function IntegrationPage() {
  return (
    <div className="max-w-4xl space-y-8">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-2">Integration Guide</h2>
        <p className="text-gray-600">
          Accept USDT (TRC-20) payments in your application with a few API calls.
        </p>
      </div>

      {/* How it Works */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">How It Works</h3>
        <div className="bg-gray-50 rounded-lg p-4 space-y-3 text-sm text-gray-700">
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-xs font-bold shrink-0">1</span>
            <div>
              <p className="font-medium">Create a payment</p>
              <p className="text-gray-500">Send amount and your order reference. Get a deposit address.</p>
            </div>
          </div>
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-xs font-bold shrink-0">2</span>
            <div>
              <p className="font-medium">Show deposit address to customer</p>
              <p className="text-gray-500">Display the TRON address and exact amount. Customer sends USDT.</p>
            </div>
          </div>
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-xs font-bold shrink-0">3</span>
            <div>
              <p className="font-medium">Wait for confirmation</p>
              <p className="text-gray-500">We detect the transaction on-chain and wait for 20 confirmations.</p>
            </div>
          </div>
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-xs font-bold shrink-0">4</span>
            <div>
              <p className="font-medium">Get notified via webhook</p>
              <p className="text-gray-500">We POST to your URL when payment is confirmed.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Setup */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Setup</h3>
        <ol className="space-y-2 text-sm text-gray-600 list-decimal list-inside">
          <li>Create a CryptoPay account at cryptopay.com</li>
          <li>Copy your API Key from the dashboard</li>
          <li>Set your withdrawal TRON address (T...) in Settings</li>
          <li>Set your webhook URL in Settings (optional but recommended)</li>
        </ol>
      </section>

      {/* Authentication */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Authentication</h3>
        <p className="text-sm text-gray-600 mb-3">
          Include your API key in every request header:
        </p>
        <CodeBlock
          title="Header"
          code={`X-API-Key: cpay_live_xxxxxxxxxxxxxxxx`}
        />
      </section>

      {/* Create Payment */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Create Payment</h3>
        <p className="text-sm text-gray-600 mb-3">
          Generate a unique deposit address for each payment:
        </p>
        <CodeBlock
          title="Request"
          code={`POST ${apiBaseUrl}/v1/payments
Content-Type: application/json
X-API-Key: cpay_live_xxxxxxxxxxxxxxxx

{
  "amount": "25.00",
  "currency": "USDT",
  "idempotency_key": "order_12345"
}`}
        />
        <div className="mt-3 p-3 bg-gray-50 rounded-lg border border-gray-200 text-sm">
          <p className="font-medium text-gray-700 mb-2">Response:</p>
          <pre className="text-gray-600 font-mono text-xs">{`{
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "deposit_address": "TNPeeaaFB7xK1nF4Kkq1Jf1B6aMt8Xc3Z4",
  "amount": "25.00",
  "currency": "USDT",
  "status": "pending",
  "expires_at": "2024-01-15T11:00:00Z"
}`}</pre>
        </div>
        <Alert type="info">
          <strong>Important:</strong> Show the customer exactly <code className="bg-gray-100 px-1">amount</code> USDT to the <code className="bg-gray-100 px-1">deposit_address</code>. Any overpayment or underpayment will not match.
        </Alert>
      </section>

      {/* Get Payment Status */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Get Payment Status</h3>
        <CodeBlock
          title="Request"
          code={`GET ${apiBaseUrl}/v1/payments/550e8400-e29b-41d4-a716-446655440000
X-API-Key: cpay_live_xxxxxxxxxxxxxxxx`}
        />
        <div className="mt-3 p-3 bg-gray-50 rounded-lg border border-gray-200 text-sm">
          <p className="font-medium text-gray-700 mb-2">Response:</p>
          <pre className="text-gray-600 font-mono text-xs">{`{
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "amount": "25.00",
  "currency": "USDT",
  "status": "confirmed",
  "tx_hash": "abc123def456...",
  "confirmations": 20,
  "created_at": "2024-01-15T10:30:00Z",
  "confirmed_at": "2024-01-15T10:35:00Z"
}`}</pre>
        </div>
        <p className="text-sm text-gray-500 mt-2">
          <strong>Status values:</strong> <code className="bg-gray-100 px-1">pending</code> → <code className="bg-gray-100 px-1">detected</code> → <code className="bg-gray-100 px-1">confirming</code> → <code className="bg-gray-100 px-1">confirmed</code>
        </p>
      </section>

      {/* Webhooks */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Webhooks</h3>
        <p className="text-sm text-gray-600 mb-3">
          Set a webhook URL in Settings to receive instant payment notifications:
        </p>
        <CodeBlock
          title="Webhook Payload"
          code={`POST https://yourapp.com/webhooks/cryptopay
Content-Type: application/json
X-CryptoPay-Signature: sha256=abc123...

{
  "event": "payment.confirmed",
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "amount": "25.00",
  "currency": "USDT",
  "tx_hash": "abc123def456...",
  "confirmations": 20,
  "confirmed_at": "2024-01-15T10:35:00Z"
}`}
        />
        <p className="text-sm text-gray-600 mt-3 mb-2">Verify the signature:</p>
        <CodeBlock
          title="Node.js Example"
          code={`const crypto = require('crypto');

app.post('/webhooks/cryptopay', (req, res) => {
  const signature = req.headers['x-cryptopay-signature'];
  const payload = JSON.stringify(req.body);
  const expected = 'sha256=' + crypto
    .createHmac('sha256', process.env.WEBHOOK_SECRET)
    .update(payload)
    .digest('hex');

  if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected))) {
    return res.status(401).send('Invalid signature');
  }

  // Payment confirmed - fulfill order
  const { payment_id, amount } = req.body;
  // Your logic here...

  res.status(200).send('OK');
});`}
        />
        <Alert type="warning">
          <strong>Always</strong> verify the signature before processing. Return HTTP 200 within 5 seconds to acknowledge receipt.
        </Alert>
      </section>

      {/* Full Example */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Full Example</h3>
        <CodeBlock
          title="Node.js - Create Payment & Handle Webhook"
          code={`const express = require('express');
const crypto = require('crypto');
const app = express();

app.use(express.json());

// 1. Create payment when customer checks out
app.post('/create-payment', async (req, res) => {
  const { amount, orderId } = req.body;

  const response = await fetch('${apiBaseUrl}/v1/payments', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': process.env.CRYPTOPAY_API_KEY
    },
    body: JSON.stringify({
      amount,
      currency: 'USDT',
      idempotency_key: orderId
    })
  });

  const payment = await response.json();
  // Show payment.deposit_address and payment.amount to customer
  res.json({ depositAddress: payment.deposit_address, amount: payment.amount });
});

// 2. Receive webhook when payment is confirmed
app.post('/webhooks/cryptopay', (req, res) => {
  const signature = req.headers['x-cryptopay-signature'];
  const payload = JSON.stringify(req.body);
  const expected = 'sha256=' + crypto
    .createHmac('sha256', process.env.WEBHOOK_SECRET)
    .update(payload)
    .digest('hex');

  if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected))) {
    return res.status(401).send('Invalid signature');
  }

  if (req.body.event === 'payment.confirmed') {
    const { payment_id, amount } = req.body;
    // Update order status to "paid" in your database
    console.log(\`Payment \${payment_id} confirmed: \${amount} USDT\`);
  }

  res.status(200).send('OK');
});

app.listen(3000);`}
        />
      </section>

      {/* Idempotency */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Idempotency</h3>
        <p className="text-sm text-gray-600 mb-3">
          To prevent duplicate payments, pass a unique <code className="bg-gray-100 px-1">idempotency_key</code> when creating a payment. If the same key is used again, the original payment is returned without creating a new one.
        </p>
        <CodeBlock
          title="Example"
          code={`POST /v1/payments
{
  "amount": "25.00",
  "currency": "USDT",
  "idempotency_key": "order_12345"
}

// If order_12345 was already used, returns the original payment
// instead of creating a duplicate`}
        />
      </section>

      {/* Error Handling */}
      <section>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Error Handling</h3>
        <div className="space-y-2 text-sm">
          <div className="flex items-center gap-2">
            <span className="w-16 px-2 py-0.5 bg-red-100 text-red-700 rounded text-xs font-mono">400</span>
            <span className="text-gray-600">Bad request - missing or invalid parameters</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="w-16 px-2 py-0.5 bg-red-100 text-red-700 rounded text-xs font-mono">401</span>
            <span className="text-gray-600">Unauthorized - invalid API key</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="w-16 px-2 py-0.5 bg-red-100 text-red-700 rounded text-xs font-mono">404</span>
            <span className="text-gray-600">Not found - payment or merchant not found</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="w-16 px-2 py-0.5 bg-red-100 text-red-700 rounded text-xs font-mono">409</span>
            <span className="text-gray-600">Conflict - idempotency key already used with different params</span>
          </div>
        </div>
        <p className="text-sm text-gray-500 mt-3">
          Error responses include a message: <code className="bg-gray-100 px-1">{"{ \"error\": \"description\" }"}</code>
        </p>
      </section>
    </div>
  );
}