import Link from "next/link";
import {
  ArrowRight,
  BarChart3,
  Clock,
  Rocket,
  Shield,
  Sparkles,
} from "lucide-react";

import { Button } from "@/components/ui/button";

export default function LandingPage() {
  return (
    <>
      <header className="border-b border-white/5">
        <div className="container flex h-16 items-center justify-between px-4 md:px-6">
          <Link href="/" className="flex items-center gap-2 font-semibold">
            <span className="text-2xl">🪼</span>
            <span className="text-xl tracking-tight">TreasuryFlow</span>
          </Link>
          <nav className="hidden gap-6 md:flex">
            <Link
              href="#features"
              className="text-sm font-medium text-muted-foreground transition-colors hover:text-primary"
            >
              Features
            </Link>
            <Link
              href="#about"
              className="text-sm font-medium text-muted-foreground transition-colors hover:text-primary"
            >
              About
            </Link>
          </nav>
          <div className="flex gap-4">
            <Link href="/dashboard">
              <Button variant="ghost" size="sm">
                Sign In
              </Button>
            </Link>
            <Link href="/create-treasury">
              <Button
                size="sm"
                className="primary-gradient hover:primary-gradient-hover glow"
              >
                Get Started
              </Button>
            </Link>
          </div>
        </div>
      </header>
      <div className="flex flex-col">
        <main className="flex-1">
          <section className="w-full py-24 md:py-32 gradient-bg">
            <div className="container relative z-10 flex flex-col items-center justify-center gap-4 px-4 text-center md:px-6">
              <div className="inline-flex items-center rounded-full border border-white/10 bg-black/20 backdrop-blur-sm px-3 py-1 text-sm">
                <Sparkles className="mr-1 h-3.5 w-3.5 text-purple-500 animate-pulse-glow" />
                <span>Introducing TreasuryFlow</span>
              </div>
              <div className="space-y-3">
                <h1 className="text-4xl font-bold tracking-tighter sm:text-5xl md:text-6xl">
                  Manage Your Treasury with{" "}
                  <span className="primary-text-gradient">Ease</span>
                </h1>
                <p className="mx-auto max-w-[700px] text-muted-foreground md:text-xl">
                  Streamline your treasury operations, track payouts, and manage
                  funds all in one place.
                </p>
              </div>
              <div className="flex flex-col gap-2 min-[400px]:flex-row">
                <Link href="/create-treasury">
                  <Button
                    size="lg"
                    className="gap-1.5 primary-gradient hover:primary-gradient-hover glow"
                  >
                    Create Treasury
                    <ArrowRight className="h-4 w-4" />
                  </Button>
                </Link>
                <Link href="/dashboard">
                  <Button
                    size="lg"
                    variant="outline"
                    className="border-white/10 bg-black/20 backdrop-blur-sm hover:bg-black/40"
                  >
                    View Demo
                  </Button>
                </Link>
              </div>
            </div>
          </section>
          <section id="features" className="w-full py-12 md:py-24 grid-pattern">
            <div className="container px-4 md:px-6">
              <div className="flex flex-col items-center justify-center gap-4 text-center">
                <div className="space-y-2">
                  <h2 className="text-3xl font-bold tracking-tighter sm:text-4xl md:text-5xl">
                    Features
                  </h2>
                  <p className="mx-auto max-w-[700px] text-muted-foreground md:text-xl">
                    Everything you need to manage your treasury efficiently.
                  </p>
                </div>
              </div>
              <div className="mx-auto grid max-w-5xl grid-cols-1 gap-6 py-12 md:grid-cols-3">
                <div className="flex flex-col items-center gap-2 rounded-lg border border-white/5 bg-black/20 p-6 shadow-sm backdrop-blur-sm">
                  <div className="rounded-full feature-gradient-1 p-3">
                    <BarChart3 className="h-6 w-6 text-rose-500" />
                  </div>
                  <h3 className="text-xl font-bold">Real-time Analytics</h3>
                  <p className="text-center text-muted-foreground">
                    Track your treasury performance with real-time analytics and
                    insights.
                  </p>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border border-white/5 bg-black/20 p-6 shadow-sm backdrop-blur-sm">
                  <div className="rounded-full feature-gradient-2 p-3">
                    <Clock className="h-6 w-6 text-blue-500" />
                  </div>
                  <h3 className="text-xl font-bold">Scheduled Payouts</h3>
                  <p className="text-center text-muted-foreground">
                    Set up recurring payouts and never miss a payment deadline.
                  </p>
                </div>
                <div className="flex flex-col items-center gap-2 rounded-lg border border-white/5 bg-black/20 p-6 shadow-sm backdrop-blur-sm">
                  <div className="rounded-full feature-gradient-3 p-3">
                    <Shield className="h-6 w-6 text-green-500" />
                  </div>
                  <h3 className="text-xl font-bold">Secure Management</h3>
                  <p className="text-center text-muted-foreground">
                    Keep your treasury secure with advanced security features
                    and permissions.
                  </p>
                </div>
              </div>
            </div>
          </section>
          <section className="w-full py-12 md:py-24 gradient-bg dot-pattern">
            <div className="container relative z-10 px-4 md:px-6">
              <div className="flex flex-col items-center justify-center gap-4 text-center">
                <div className="inline-flex items-center rounded-full border border-white/10 bg-black/20 backdrop-blur-sm px-3 py-1 text-sm">
                  <Rocket className="mr-1 h-3.5 w-3.5 text-purple-500 animate-pulse-glow" />
                  <span>Get Started Today</span>
                </div>
                <div className="space-y-2">
                  <h2 className="text-3xl font-bold tracking-tighter sm:text-4xl md:text-5xl">
                    Ready to{" "}
                    <span className="primary-text-gradient">Transform</span>{" "}
                    Your Treasury?
                  </h2>
                  <p className="mx-auto max-w-[700px] text-muted-foreground md:text-xl">
                    Create your treasury in minutes and start managing your
                    funds efficiently.
                  </p>
                </div>
                <Link href="/create-treasury">
                  <Button
                    size="lg"
                    className="mt-4 primary-gradient hover:primary-gradient-hover glow"
                  >
                    Create Treasury
                  </Button>
                </Link>
              </div>
            </div>
          </section>
        </main>
      </div>
    </>
  );
}
