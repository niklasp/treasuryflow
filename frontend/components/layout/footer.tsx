import Link from "next/link";
import { ThemeToggle } from "@/components/ui/theme-toggle";

export function Footer() {
  return (
    <footer className="border-t">
      <div className="container flex flex-col gap-6 py-8 md:flex-row md:items-center md:justify-between md:py-12">
        <div className="flex items-center gap-2">
          <span className="text-2xl">🪼</span>
          <span className="font-semibold">TreasuryFlow</span>
        </div>
        <p className="text-sm text-muted-foreground">
          © 2025 TreasuryFlow. All rights reserved.
        </p>
        <div className="flex items-center gap-4">
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
          <ThemeToggle />
        </div>
      </div>
    </footer>
  );
}
