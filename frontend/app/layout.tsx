import type React from "react";
import type { Metadata } from "next";
import Link from "next/link";
import { Manrope } from "next/font/google";

import { Button } from "@/components/ui/button";
import "./globals.css";

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
    <html lang="en" className="dark">
      <body
        className={`${manrope.variable} font-sans flex flex-col min-h-screen`}
      >
        <div className="flex-1">{children}</div>
        <footer className="border-t border-white/5">
          <div className="container flex flex-col gap-6 py-8 md:flex-row md:items-center md:justify-between md:py-12">
            <div className="flex items-center gap-2">
              <span className="text-2xl">🪼</span>
              <span className="font-semibold">TreasuryFlow</span>
            </div>
            <p className="text-sm text-muted-foreground">
              © 2025 TreasuryFlow. All rights reserved.
            </p>
            <nav className="flex gap-4">
              <Link
                href="#"
                className="text-sm text-muted-foreground transition-colors hover:text-primary"
              >
                Terms
              </Link>
              <Link
                href="#"
                className="text-sm text-muted-foreground transition-colors hover:text-primary"
              >
                Privacy
              </Link>
              <Link
                href="#"
                className="text-sm text-muted-foreground transition-colors hover:text-primary"
              >
                Contact
              </Link>
            </nav>
          </div>
        </footer>
      </body>
    </html>
  );
}
