import Link from "next/link";
import { HugeiconsIcon } from "@hugeicons/react";
import type { IconSvgElement } from "@hugeicons/react";
import { ArrowRight01Icon } from "@/lib/icons";

interface CtaButtonProps {
  href: string;
  label: string;
  size?: "sm" | "md";
  icon?: IconSvgElement;
  className?: string;
}

const SIZE = {
  sm: { padding: "px-4 py-2", icon: 16, lift: "hover:-translate-y-px", nudge: "group-hover:translate-x-0.5" },
  md: { padding: "px-7 py-3.5", icon: 18, lift: "hover:-translate-y-0.5", nudge: "group-hover:translate-x-1" },
} as const;

export function CtaButton({ href, label, size = "md", icon = ArrowRight01Icon, className }: CtaButtonProps) {
  const variant = SIZE[size];
  return (
    <Link
      href={href}
      className={`group flex items-center justify-center gap-2 rounded-full bg-gradient-to-b from-accent-strong to-accent text-sm font-semibold text-background btn-gloss transition-transform ${variant.padding} ${variant.lift} ${className ?? ""}`}
    >
      {label}
      <HugeiconsIcon
        icon={icon}
        size={variant.icon}
        className={`transition-transform ${variant.nudge}`}
      />
    </Link>
  );
}
