"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "./theme-provider";
import { ExtensionProvider } from "./polkadot-extension-provider";
import { LightClientApiProvider } from "./lightclient-api-provider";
import { useState } from "react";

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
    <QueryClientProvider client={queryClient}>
      <ThemeProvider defaultTheme="dark">
        <ExtensionProvider>
          {children}
          {/* <LightClientApiProvider>{children}</LightClientApiProvider> */}
        </ExtensionProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}
