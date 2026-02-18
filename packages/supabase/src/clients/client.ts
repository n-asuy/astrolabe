import { createClient as createSupabaseClient } from "@supabase/supabase-js";
import type { Database } from "../types/db";

let client: ReturnType<typeof createSupabaseClient<Database>> | null = null;

export function createClient(url: string, anonKey: string) {
  if (client) return client;

  if (!url || !anonKey) {
    throw new Error(
      "Supabase is not configured. Set VITE_SUPABASE_URL and VITE_SUPABASE_ANON_KEY in .env",
    );
  }

  client = createSupabaseClient<Database>(url, anonKey);
  return client;
}
