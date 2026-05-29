import Link from "next/link";
import { HugeiconsIcon } from "@hugeicons/react";
import { NAV_LINKS, PRIMARY_CTA, SITE } from "@/lib/content";
import { ArrowRight01Icon } from "@/lib/icons";

export function SiteNav() {
  return (
    <header className="fixed inset-x-0 top-4 z-50 flex justify-center px-4">
      <nav className="glass-panel flex items-center gap-1 rounded-full p-1.5 pl-5">
        <Link
          href="/"
          className="font-display text-sm font-bold tracking-[0.28em] text-foreground transition-colors hover:text-accent"
        >
          {SITE.wordmark}
        </Link>

        <span className="mx-3 hidden h-5 w-px bg-white/10 sm:block" />

        <ul className="hidden items-center gap-1 md:flex">
          {NAV_LINKS.map((link) => (
            <li key={link.href}>
              <Link
                href={link.href}
                className="rounded-full px-3.5 py-2 text-sm text-muted transition-colors hover:bg-white/[0.04] hover:text-foreground"
              >
                {link.label}
              </Link>
            </li>
          ))}
        </ul>

        <Link
          href={PRIMARY_CTA.href}
          className="group ml-2 flex items-center gap-1.5 rounded-full bg-gradient-to-b from-accent-strong to-accent px-4 py-2 text-sm font-semibold text-background btn-gloss transition-transform hover:-translate-y-px"
        >
          {PRIMARY_CTA.label}
          <HugeiconsIcon
            icon={ArrowRight01Icon}
            size={16}
            className="transition-transform group-hover:translate-x-0.5"
          />
        </Link>
      </nav>
    </header>
  );
}
