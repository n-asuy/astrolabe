import { createClient } from "@astrolabe/supabase/client";

let _client: ReturnType<typeof createClient> | null = null;
let _error: string | null = null;

export function getSupabase() {
  if (_client) return _client;
  if (_error) throw new Error(_error);

  try {
    _client = createClient(
      import.meta.env.VITE_SUPABASE_URL,
      import.meta.env.VITE_SUPABASE_ANON_KEY,
    );
    return _client;
  } catch (e) {
    _error = e instanceof Error ? e.message : "Failed to initialize Supabase";
    throw e;
  }
}

export function getSupabaseError(): string | null {
  if (_client) return null;
  try {
    getSupabase();
    return null;
  } catch (e) {
    return e instanceof Error ? e.message : "Failed to initialize Supabase";
  }
}
