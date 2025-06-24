"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider as NextThemesProvider } from "next-themes";
import { ExtensionProvider } from "./polkadot-extension-provider";
import { useState } from "react";
import { ConvexClientProvider } from "./convex-provider";

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 1000 * 60 * 5, // 5 minutes
            refetchOnWindowFocus: false,
          },
        },
      })
  );

  return (
    <ConvexClientProvider>
      <QueryClientProvider client={queryClient}>
        <NextThemesProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange={false}
          storageKey="theme"
        >
          <ExtensionProvider>
            {children}
            {/* <LightClientApiProvider>{children}</LightClientApiProvider> */}
          </ExtensionProvider>
        </NextThemesProvider>
      </QueryClientProvider>
    </ConvexClientProvider>
  );
}
