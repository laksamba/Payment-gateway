import { useState } from "react";
import { useAuth } from "../hooks/useAuth";
import { DashboardLayout } from "../components/DashboardLayout";
import { PaymentsPage } from "./PaymentsPage";
import { SettingsPage } from "./SettingsPage";
import { Navigate } from "react-router-dom";

export function DashboardPage() {
  const { apiKey, profile, logout } = useAuth();
  const [activeTab, setActiveTab] = useState("payments");

  if (!apiKey) return <Navigate to="/login" replace />;

  return (
    <DashboardLayout
      activeTab={activeTab}
      onTabChange={setActiveTab}
      onLogout={logout}
      merchantName={profile?.name}
    >
      {activeTab === "payments" ? <PaymentsPage /> : <SettingsPage />}
    </DashboardLayout>
  );
}
