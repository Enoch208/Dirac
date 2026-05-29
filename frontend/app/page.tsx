import { SiteNav } from "@/components/site-nav";
import { Hero } from "@/components/hero";
import { HowItWorks } from "@/components/how-it-works";
import { FeatureBento } from "@/components/feature-bento";
import { PotBanner } from "@/components/pot-banner";
import { SiteFooter } from "@/components/site-footer";

export default function Home() {
  return (
    <>
      <SiteNav />
      <main>
        <Hero />
        <HowItWorks />
        <FeatureBento />
        <PotBanner />
      </main>
      <SiteFooter />
    </>
  );
}
