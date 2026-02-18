import type { User } from "@supabase/supabase-js";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import { getSupabase, getSupabaseError } from "./supabase";

interface AuthContextValue {
  user: User | null;
  loading: boolean;
  error: string | null;
  signOut: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue>({
  user: null,
  loading: true,
  error: null,
  signOut: async () => {},
});

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const configError = getSupabaseError();
    if (configError) {
      setError(configError);
      setLoading(false);
      return;
    }

    const supabase = getSupabase();

    supabase.auth
      .getUser()
      .then(({ data, error: authError }) => {
        if (authError) {
          // Not signed in is not an error state
          setUser(null);
        } else {
          setUser(data.user);
        }
        setLoading(false);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : "Authentication failed");
        setLoading(false);
      });

    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((_event, session) => {
      setUser(session?.user ?? null);
      setLoading(false);
    });

    return () => subscription.unsubscribe();
  }, []);

  const signOut = useCallback(async () => {
    try {
      const supabase = getSupabase();
      await supabase.auth.signOut();
      setUser(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Sign out failed");
    }
  }, []);

  return (
    <AuthContext.Provider value={{ user, loading, error, signOut }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
