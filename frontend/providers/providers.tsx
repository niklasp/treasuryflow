"use client";

import { ThemeProvider } from "./theme-provider";
import { ExtensionProvider } from "./polkadot-extension-provider";
import { RpcApiProvider } from "./rpc-api-provider";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider defaultTheme="dark">
      <ExtensionProvider>
        <RpcApiProvider>{children}</RpcApiProvider>
      </ExtensionProvider>
    </ThemeProvider>
  );
}
