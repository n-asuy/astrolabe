import { Button } from "@astrolabe/ui/button";
import { Icons } from "@astrolabe/ui/icons";
import { useAuth } from "~/lib/auth";

export default function HomePage() {
  const { user, signOut } = useAuth();

  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center">
      <div className="flex flex-col items-center justify-center gap-4">
        <p>Welcome, {user?.email}</p>
        <Button onClick={signOut} variant="outline" className="gap-2 font-mono">
          <Icons.SignOut className="size-4" />
          <span>Sign out</span>
        </Button>
      </div>
    </div>
  );
}
