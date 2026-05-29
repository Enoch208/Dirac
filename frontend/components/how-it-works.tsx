import { HugeiconsIcon } from "@hugeicons/react";
import { STEPS } from "@/lib/content";
import { Reveal } from "./reveal";

export function HowItWorks() {
  return (
    <section id="how" className="relative mx-auto max-w-6xl px-6 py-28">
      <div className="pointer-events-none absolute left-1/2 top-0 -z-10 h-[360px] w-[700px] -translate-x-1/2 rounded-full bg-[radial-gradient(ellipse_at_center,rgba(245,181,68,0.06),transparent_70%)] blur-3xl" />

      <Reveal className="mx-auto max-w-2xl text-center">
        <p className="font-mono text-xs uppercase tracking-[0.22em] text-accent">
          The loop
        </p>
        <h2 className="mt-4 font-display text-4xl font-semibold tracking-tight sm:text-5xl">
          One call in. A ranked duel out.
        </h2>
      </Reveal>

      <div className="mt-16 grid gap-px overflow-hidden rounded-2xl border border-white/5 bg-white/5 sm:grid-cols-2 lg:grid-cols-4">
        {STEPS.map((step, index) => (
          <Reveal key={step.index} delay={index + 1} className="h-full">
            <article className="flex h-full flex-col gap-5 bg-background p-7">
              <div className="flex items-center justify-between">
                <HugeiconsIcon
                  icon={step.icon}
                  size={28}
                  strokeWidth={1.5}
                  className="text-accent"
                />
                <span className="font-display text-2xl font-semibold text-white/8">
                  {step.index}
                </span>
              </div>
              <h3 className="font-display text-lg font-semibold">{step.title}</h3>
              <p className="text-sm leading-relaxed text-muted">{step.body}</p>
            </article>
          </Reveal>
        ))}
      </div>
    </section>
  );
}
