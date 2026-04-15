import { Link } from "react-router-dom";
import {
  Zap,
  Shield,
  Globe,
  ArrowRight,
  CheckCircle,
  TrendingUp,
  Users,
  CreditCard,
  Bitcoin,
  Lock,
  RefreshCw,
} from "lucide-react";

function FeatureCard({
  icon: Icon,
  title,
  description,
}: {
  icon: React.ElementType;
  title: string;
  description: string;
}) {
  return (
    <div className="p-6 rounded-2xl bg-white border border-gray-100 shadow-sm hover:shadow-md transition-shadow">
      <div className="w-12 h-12 bg-green-100 rounded-xl flex items-center justify-center mb-4">
        <Icon className="w-6 h-6 text-green-600" />
      </div>
      <h3 className="text-lg font-semibold text-gray-900 mb-2">{title}</h3>
      <p className="text-gray-600 text-sm leading-relaxed">{description}</p>
    </div>
  );
}

export function LandingPage() {
  return (
    <div className="min-h-screen bg-white">
      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 bg-white/80 backdrop-blur-md z-50 border-b border-gray-100">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-2">
              <div className="w-9 h-9 bg-green-600 rounded-lg flex items-center justify-center">
                <TrendingUp className="w-5 h-5 text-white" />
              </div>
              <span className="text-xl font-bold text-gray-900">CryptoPay</span>
            </div>
            <div className="flex items-center gap-4">
              <Link
                to="/login"
                className="px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 transition-colors"
              >
                Sign In
              </Link>
              <Link
                to="/login"
                className="px-4 py-2 text-sm font-medium bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
              >
                Get Started
              </Link>
            </div>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="pt-32 pb-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-b from-green-50 to-white">
        <div className="max-w-7xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 px-4 py-2 bg-green-100 rounded-full text-green-700 text-sm font-medium mb-6">
            <Zap className="w-4 h-4" />
            Accept Crypto Payments in Minutes
          </div>
          <h1 className="text-5xl sm:text-6xl font-bold text-gray-900 mb-6 leading-tight">
            Accept USDT Payments
            <br />
            <span className="text-green-600">Without the Complexity</span>
          </h1>
          <p className="text-xl text-gray-600 mb-10 max-w-2xl mx-auto">
            The simplest way to accept TRC-20 USDT payments. No custody, no risk.
            Your funds go directly to your wallet.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              to="/login"
              className="w-full sm:w-auto px-8 py-4 bg-green-600 text-white font-semibold rounded-xl hover:bg-green-700 transition-colors flex items-center justify-center gap-2"
            >
              Start Accepting Payments <ArrowRight className="w-5 h-5" />
            </Link>
            <Link
              to="/login"
              className="w-full sm:w-auto px-8 py-4 bg-white text-gray-900 font-semibold rounded-xl border border-gray-200 hover:border-gray-300 transition-colors"
            >
              View Documentation
            </Link>
          </div>
          <p className="text-sm text-gray-500 mt-6">No credit card required • Setup in 2 minutes</p>
        </div>
      </section>

      {/* Stats */}
      <section className="py-12 bg-gray-900">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {[
              { label: "Transaction Fee", value: "1%" },
              { label: "Settlement Time", value: "~5 min" },
              { label: "Network", value: "TRON" },
              { label: "Currencies", value: "USDT" },
            ].map((stat) => (
              <div key={stat.label} className="text-center">
                <div className="text-3xl font-bold text-white mb-1">{stat.value}</div>
                <div className="text-gray-400 text-sm">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">
              Everything You Need to Accept Crypto
            </h2>
            <p className="text-lg text-gray-600 max-w-2xl mx-auto">
              Built for developers and businesses. Simple API, powerful features,
              zero custody.
            </p>
          </div>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            <FeatureCard
              icon={Zap}
              title="Instant Settlements"
              description="Funds are sent directly to your wallet. No waiting, no custody risk. Just pure, direct payments."
            />
            <FeatureCard
              icon={Shield}
              title="Non-Custodial"
              description="We never hold your funds. Your money goes directly from the customer to your wallet."
            />
            <FeatureCard
              icon={Globe}
              title="Global Payments"
              description="Accept payments from anywhere in the world. No borders, no restrictions."
            />
            <FeatureCard
              icon={CreditCard}
              title="Simple Integration"
              description="REST API with clear documentation. Get started in minutes, not days."
            />
            <FeatureCard
              icon={RefreshCw}
              title="Automatic Conversions"
              description="Receive USDT, withdraw USDT. No volatility risk, no complex DeFi."
            />
            <FeatureCard
              icon={Lock}
              title="Secure by Design"
              description="Enterprise-grade security. HMAC signatures, encrypted connections, battle-tested infrastructure."
            />
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gray-50">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">How It Works</h2>
            <p className="text-lg text-gray-600">
              Accept USDT payments in three simple steps
            </p>
          </div>
          <div className="grid md:grid-cols-3 gap-8">
            <div className="bg-white rounded-2xl p-8 shadow-sm">
              <div className="w-10 h-10 bg-green-100 rounded-xl flex items-center justify-center mb-6">
                <span className="text-green-600 font-bold">1</span>
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-3">
                Create Payment
              </h3>
              <p className="text-gray-600 text-sm mb-4">
                Call our API with the amount and get a unique deposit address instantly.
              </p>
              <div className="bg-gray-900 rounded-lg p-4 font-mono text-xs text-green-400">
                POST /v1/payments
                <br />
                {"{ amount: \"100\" }"}
              </div>
            </div>
            <div className="bg-white rounded-2xl p-8 shadow-sm">
              <div className="w-10 h-10 bg-green-100 rounded-xl flex items-center justify-center mb-6">
                <span className="text-green-600 font-bold">2</span>
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-3">
                Customer Pays
              </h3>
              <p className="text-gray-600 text-sm mb-4">
                Share the deposit address with your customer. They send USDT (TRC-20) to it.
              </p>
              <div className="flex items-center gap-2 text-sm text-gray-600">
                <CheckCircle className="w-4 h-4 text-green-600" />
                Exact amount matching
              </div>
            </div>
            <div className="bg-white rounded-2xl p-8 shadow-sm">
              <div className="w-10 h-10 bg-green-100 rounded-xl flex items-center justify-center mb-6">
                <span className="text-green-600 font-bold">3</span>
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-3">
                Get Notified
              </h3>
              <p className="text-gray-600 text-sm mb-4">
                Receive a webhook when payment is confirmed. 20 blockchain confirmations.
              </p>
              <div className="bg-gray-900 rounded-lg p-4 font-mono text-xs text-green-400">
                payment.confirmed
                <br />
                tx_hash: 0x...
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Supported */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="grid md:grid-cols-2 gap-12 items-center">
            <div>
              <h2 className="text-3xl font-bold text-gray-900 mb-6">
                Built for TRON Network
              </h2>
              <p className="text-lg text-gray-600 mb-8">
                TRON offers fast confirmations and near-zero fees, making it
                perfect for payment applications.
              </p>
              <div className="space-y-4">
                {[
                  { icon: Bitcoin, text: "Fast confirmations (~3 seconds)" },
                  { icon: Zap, text: "Near-zero transaction fees" },
                  { icon: Globe, text: "Eco-friendly, energy efficient" },
                  { icon: Users, text: "Millions of TRON users" },
                ].map((item) => (
                  <div key={item.text} className="flex items-center gap-3">
                    <div className="w-8 h-8 bg-green-100 rounded-lg flex items-center justify-center">
                      <item.icon className="w-4 h-4 text-green-600" />
                    </div>
                    <span className="text-gray-700">{item.text}</span>
                  </div>
                ))}
              </div>
            </div>
            <div className="bg-gradient-to-br from-green-50 to-emerald-50 rounded-3xl p-8">
              <div className="bg-white rounded-2xl p-6 shadow-lg">
                <div className="flex items-center gap-3 mb-6">
                  <div className="w-10 h-10 bg-green-600 rounded-full flex items-center justify-center">
                    <svg className="w-6 h-6 text-white" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                    </svg>
                  </div>
                  <div>
                    <div className="font-semibold text-gray-900">USDT TRC-20</div>
                    <div className="text-sm text-gray-500">Tether on TRON Network</div>
                  </div>
                </div>
                <div className="space-y-3">
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-gray-600 text-sm">Confirmations</span>
                    <span className="font-medium text-gray-900">20</span>
                  </div>
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-gray-600 text-sm">Avg. Confirmation</span>
                    <span className="font-medium text-gray-900">~3 sec</span>
                  </div>
                  <div className="flex justify-between items-center py-2">
                    <span className="text-gray-600 text-sm">Network Fee</span>
                    <span className="font-medium text-green-600">~$0.001</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Security */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gray-900">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">
              Enterprise-Grade Security
            </h2>
            <p className="text-lg text-gray-400">
              Your and your customers' data is protected at every step
            </p>
          </div>
          <div className="grid md:grid-cols-3 gap-6">
            {[
              {
                title: "Signed Webhooks",
                description: "Every webhook is signed with HMAC-SHA256. Verify every payload before processing.",
              },
              {
                title: "Encrypted Connections",
                description: "All API requests use TLS encryption. Your data is never transmitted in plain text.",
              },
              {
                title: "API Key Authentication",
                description: "Secure API key authentication with SHA-256 hashing. Keys are never stored in plain text.",
              },
            ].map((item) => (
              <div key={item.title} className="bg-gray-800 rounded-2xl p-6 border border-gray-700">
                <div className="w-10 h-10 bg-green-600/20 rounded-xl flex items-center justify-center mb-4">
                  <Shield className="w-5 h-5 text-green-500" />
                </div>
                <h3 className="text-lg font-semibold text-white mb-2">{item.title}</h3>
                <p className="text-gray-400 text-sm">{item.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Pricing */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">
              Simple, Transparent Pricing
            </h2>
            <p className="text-lg text-gray-600">
              No hidden fees. No monthly fees. Pay only for what you use.
            </p>
          </div>
          <div className="max-w-md mx-auto">
            <div className="bg-gradient-to-br from-green-600 to-emerald-600 rounded-3xl p-8 text-white">
              <div className="text-center mb-6">
                <div className="text-5xl font-bold mb-2">1%</div>
                <div className="text-green-100">per successful transaction</div>
              </div>
              <ul className="space-y-3 mb-8">
                {[
                  "No setup fees",
                  "No monthly fees",
                  "No hidden costs",
                  "Instant settlements",
                  "Unlimited transactions",
                ].map((item) => (
                  <li key={item} className="flex items-center gap-2">
                    <CheckCircle className="w-5 h-5 text-green-200" />
                    <span>{item}</span>
                  </li>
                ))}
              </ul>
              <Link
                to="/login"
                className="block w-full py-3 bg-white text-green-600 font-semibold rounded-xl text-center hover:bg-green-50 transition-colors"
              >
                Get Started Free
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-green-50">
        <div className="max-w-3xl mx-auto text-center">
          <h2 className="text-3xl font-bold text-gray-900 mb-4">
            Ready to Accept USDT Payments?
          </h2>
          <p className="text-lg text-gray-600 mb-8">
            Join thousands of businesses accepting crypto payments. Get started in minutes.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              to="/login"
              className="w-full sm:w-auto px-8 py-4 bg-green-600 text-white font-semibold rounded-xl hover:bg-green-700 transition-colors flex items-center justify-center gap-2"
            >
              Create Free Account <ArrowRight className="w-5 h-5" />
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 px-4 sm:px-6 lg:px-8 bg-gray-900">
        <div className="max-w-7xl mx-auto">
          <div className="flex flex-col md:flex-row items-center justify-between gap-6">
            <div className="flex items-center gap-2">
              <div className="w-9 h-9 bg-green-600 rounded-lg flex items-center justify-center">
                <TrendingUp className="w-5 h-5 text-white" />
              </div>
              <span className="text-xl font-bold text-white">CryptoPay</span>
            </div>
            <div className="flex items-center gap-6">
              <Link
                to="/login"
                className="text-gray-400 hover:text-white text-sm transition-colors"
              >
                Dashboard
              </Link>
              <Link
                to="/login"
                className="text-gray-400 hover:text-white text-sm transition-colors"
              >
                Documentation
              </Link>
              <a
                href="#"
                className="text-gray-400 hover:text-white text-sm transition-colors"
              >
                Support
              </a>
            </div>
            <div className="text-gray-400 text-sm">
              &copy; 2026 CryptoPay. All rights reserved.
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
