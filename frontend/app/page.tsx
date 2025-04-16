import { fontUnbounded } from "@/fonts";
import { cn } from "@/lib/utils";
import { AccountBalance } from "@/components/account/account-balance";

export default async function Home() {
  return (
    <main className="flex min-h-screen p-6 sm:p-8 pb-20 flex-col gap-[32px] row-start-2 items-center justify-center relative">
      <h1
        className={cn(
          "text-5xl lg:text-6xl bg-clip-text text-transparent bg-gradient-to-r from-foreground/70 via-foreground to-foreground/70",
          fontUnbounded.className,
        )}
      >
        Polkadot Next.js Starter
      </h1>
      <p>A starter project for building a Polkadot dApp with Next.js.</p>
      <a
        className="hidden sm:block"
        href="https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fniklasp%2Fpolkadot-nextjs-starter"
      >
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src="https://vercel.com/button"
          alt="Deploy with Vercel"
          width={103}
          height={32}
        />
      </a>
      <AccountBalance />
    </main>
  );
}
