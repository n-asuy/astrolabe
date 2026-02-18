import baseConfig from "@astrolabe/ui/tailwind.config";
import type { Config } from "tailwindcss";

export default {
  ...baseConfig,
  content: [
    "./app/**/*.{ts,tsx}",
    "../../packages/ui/src/**/*.{ts,tsx}",
  ],
} satisfies Config;
