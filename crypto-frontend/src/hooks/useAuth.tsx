import React, { createContext, useContext, useState, useEffect, useCallback } from "react";
import { merchantsApi, MerchantProfile, ApiError } from "../api/merchants";

interface AuthState {
  apiKey: string | null;
  profile: MerchantProfile | null;
  isLoading: boolean;
  error: string | null;
}

interface AuthContextType extends AuthState {
  login: (apiKey: string) => Promise<void>;
  logout: () => void;
  refreshProfile: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AuthState>(() => {
    const stored = localStorage.getItem("cryptopay_api_key");
    return {
      apiKey: stored,
      profile: null,
      isLoading: false,
      error: null,
    };
  });

  const refreshProfile = useCallback(async () => {
    if (!state.apiKey) return;
    setState((s) => ({ ...s, isLoading: true, error: null }));
    try {
      const api = merchantsApi(state.apiKey);
      const profile = await api.getProfile();
      setState((s) => ({ ...s, profile, isLoading: false }));
    } catch (err) {
      const message = err instanceof ApiError ? err.message : "Failed to fetch profile";
      setState((s) => ({ ...s, error: message, isLoading: false }));
    }
  }, [state.apiKey]);

  useEffect(() => {
    if (state.apiKey) {
      refreshProfile();
    }
  }, [state.apiKey, refreshProfile]);

  const login = useCallback(async (apiKey: string) => {
    setState((s) => ({ ...s, isLoading: true, error: null }));
    try {
      console.log("[Auth] Attempting login with API key:", apiKey);
      const api = merchantsApi(apiKey);
      const profile = await api.getProfile();
      console.log("[Auth] Login success, profile:", profile);
      localStorage.setItem("cryptopay_api_key", apiKey);
      setState({ apiKey, profile, isLoading: false, error: null });
    } catch (err) {
      console.log("[Auth] Login failed, error:", err);
      localStorage.removeItem("cryptopay_api_key");
      const message = err instanceof ApiError ? err.message : "Invalid API key";
      setState((s) => ({ ...s, error: message, isLoading: false }));
      throw new Error(message);
    }
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem("cryptopay_api_key");
    setState({ apiKey: null, profile: null, isLoading: false, error: null });
  }, []);

  return (
    <AuthContext.Provider value={{ ...state, login, logout, refreshProfile }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
