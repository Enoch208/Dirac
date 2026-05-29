import Link from "next/link";
import { HugeiconsIcon } from "@hugeicons/react";
import { HERO, HERO_STATS, PRIMARY_CTA, SECONDARY_CTA } from "@/lib/content";
import { ArrowRight01Icon, CodeIcon } from "@/lib/icons";
import { Reveal } from "./reveal";
import { TerminalWindow } from "./terminal-window";

const COLUMNS = Array.from({ length: 7 });

export function Hero() {
  return (
    <section className="relative overflow-hidden pt-44 pb-24">
      <div className="pointer-events-none absolute inset-0 -z-30 bg-halftone mask-edges opacity-60" />
      <div className="pointer-events-none absolute inset-0 -z-20 mx-auto grid max-w-5xl grid-cols-7 [mask-image:linear-gradient(to_bottom,transparent,#000_18%,#000_70%,transparent)]">
        {COLUMNS.map((_, index) => (
          <div key={index} className="border-l border-white/[0.045] last:border-r" />
        ))}
      </div>
      <div className="pointer-events-none absolute left-1/2 top-[6%] -z-10 h-[560px] w-[640px] -translate-x-1/2 rounded-full bg-[radial-gradient(ellipse_50%_55%_at_50%_40%,rgba(255,255,255,0.07),transparent_70%)]" />
      <div className="pointer-events-none absolute left-1/2 top-[-12%] -z-10 h-[560px] w-[1000px] -translate-x-1/2 animate-glow-pulse rounded-full bg-[radial-gradient(ellipse_at_center,rgba(245,181,68,0.13),transparent_66%)] blur-3xl" />

      <div className="relative mx-auto flex max-w-4xl flex-col items-center px-6 text-center">
        <Reveal className="flex flex-col items-center">
          <div className="glass-panel mb-8 flex items-center gap-2.5 rounded-full px-4 py-1.5 text-xs font-medium text-muted">
            <span className="size-1.5 animate-pulse rounded-full bg-win" />
            {HERO.badge}
            <span className="h-3 w-px bg-white/10" />
            <span className="text-accent-strong">{HERO.tag}</span>
          </div>

          <h1 className="font-display text-6xl font-bold leading-[0.95] tracking-[-0.03em] sm:text-7xl md:text-8xl">
            <span className="text-gradient-fade">{HERO.titleWords[0]} </span>
            <span className="text-gradient-fade">{HERO.titleWords[1]} </span>
            <span className="text-gradient-accent">{HERO.titleWords[2]}</span>
          </h1>

          <p className="mt-8 max-w-2xl text-balance text-lg leading-relaxed text-muted sm:text-xl">
            {HERO.subtitle}
          </p>

          <div className="mt-10 flex flex-col gap-3 sm:flex-row">
            <Link
              href={PRIMARY_CTA.href}
              className="group flex items-center justify-center gap-2 rounded-full bg-gradient-to-b from-accent-strong to-accent px-7 py-3.5 text-sm font-semibold text-background btn-gloss transition-transform hover:-translate-y-0.5"
            >
              {PRIMARY_CTA.label}
              <HugeiconsIcon
                icon={ArrowRight01Icon}
                size={18}
                className="transition-transform group-hover:translate-x-1"
              />
            </Link>
            <Link
              href={SECONDARY_CTA.href}
              className="btn-glass flex items-center justify-center gap-2 rounded-full px-7 py-3.5 text-sm font-medium text-foreground transition-colors hover:bg-white/[0.06]"
            >
              <HugeiconsIcon icon={CodeIcon} size={18} strokeWidth={1.6} />
              {SECONDARY_CTA.label}
            </Link>
          </div>
        </Reveal>

        <Reveal delay={2} className="mt-16 w-full max-w-2xl">
          <TerminalWindow />
        </Reveal>

        <Reveal delay={3} className="mt-12 w-full max-w-2xl">
          <dl className="glass-panel grid grid-cols-2 overflow-hidden rounded-2xl sm:grid-cols-4">
            {HERO_STATS.map((stat) => (
              <div key={stat.label} className="border-white/5 px-4 py-5 [&:not(:nth-child(4n))]:border-r">
                <dt className="font-display text-2xl font-semibold text-accent-strong">
                  {stat.value}
                </dt>
                <dd className="mt-1 text-xs uppercase tracking-[0.14em] text-faint">
                  {stat.label}
                </dd>
              </div>
            ))}
          </dl>
        </Reveal>
      </div>
    </section>
  );
}
