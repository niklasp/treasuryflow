import type React from "react";
import type { Metadata } from "next";
import Link from "next/link";
import { Manrope } from "next/font/google";

import "./globals.css";
import { Providers } from "@/providers/providers";
import { Footer } from "@/components/layout/footer";

const manrope = Manrope({ subsets: ["latin"], variable: "--font-manrope" });

export const metadata: Metadata = {
  title: "TreasuryFlow - Modern Treasury Management",
  description: "Streamline your treasury operations with TreasuryFlow",
  generator: "v0.dev",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body
        className={`${manrope.variable} font-sans flex flex-col min-h-screen`}
      >
        <Providers>
          <div className="flex-1">{children}</div>
        </Providers>
        <Footer />
      </body>
    </html>
  );
}
