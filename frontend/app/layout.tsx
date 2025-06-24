import type React from "react";
import type { Metadata } from "next";
import { Manrope } from "next/font/google";
import { Providers } from "@/providers/providers";
import { Footer } from "@/components/layout/footer";

import "../styles/globals.css";

// const manrope = Manrope({ subsets: ["latin"], variable: "--font-manrope" });

export const metadata: Metadata = {
  title: "TreasuryFlow - Modern Treasury Management",
  description: "Streamline your treasury operations with TreasuryFlow",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`antialiased`}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
