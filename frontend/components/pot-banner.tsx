import Link from "next/link";
import { HugeiconsIcon } from "@hugeicons/react";
import { POT, POT_ICON } from "@/lib/content";
import { ArrowRight01Icon } from "@/lib/icons";
import { Reveal } from "./reveal";

export function PotBanner() {
  return (
    <section id="pot" className="mx-auto max-w-6xl px-6 py-28">
      <Reveal>
        <div className="relative overflow-hidden rounded-3xl border border-accent/20 bg-gradient-to-br from-accent/[0.08] via-background to-background px-8 py-16 text-center sm:px-16">
          <div className="pointer-events-none absolute inset-0 bg-arena-grid opacity-60" />
          <div className="relative mx-auto flex max-w-2xl flex-col items-center">
            <span className="flex size-14 items-center justify-center rounded-2xl border border-accent/25 bg-accent/10 text-accent">
              <HugeiconsIcon icon={POT_ICON} size={28} strokeWidth={1.5} />
            </span>
            <p className="mt-8 font-mono text-xs uppercase tracking-[0.22em] text-accent">
              {POT.eyebrow}
            </p>
            <h2 className="mt-4 whitespace-pre-line font-display text-4xl font-semibold leading-tight tracking-tight sm:text-6xl">
              {POT.title}
            </h2>
            <p className="mt-6 text-lg leading-relaxed text-muted">{POT.body}</p>
            <Link
              href={POT.cta.href}
              className="group mt-10 flex items-center gap-2 rounded-full bg-gradient-to-b from-accent-strong to-accent px-7 py-3.5 text-sm font-semibold text-background btn-gloss transition-transform hover:-translate-y-0.5"
            >
              {POT.cta.label}
              <HugeiconsIcon
                icon={ArrowRight01Icon}
                size={18}
                className="transition-transform group-hover:translate-x-1"
              />
            </Link>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
