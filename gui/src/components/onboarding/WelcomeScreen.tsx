import { GlassCard } from "./ui/GlassCard";
import { AnimatedButton } from "./ui/AnimatedButton";
import { Logo } from "./ui/Logo";
import { ParticleBackground } from "./ui/ParticleBackground";
import { Search, Clock, Shield } from "lucide-react";
import { cn } from "@/lib/utils";
import { useState } from "react";
import { initRepo } from "@/lib/lfs";

interface WelcomeScreenProps {
  onNext: () => void;
}

const philosophyCards = [
  {
    icon: Search,
    title: "Finde durch Bedeutung",
    description: "Dateien leben nicht in Ordnern — sie leben in Kontext.",
  },
  {
    icon: Clock,
    title: "Alles erinnert sich",
    description: "Jede Änderung, jeder Zugriff, jede Verbindung wird bewahrt.",
  },
  {
    icon: Shield,
    title: "Teile präzise",
    description: "Niemals versehentlich. Immer kontrolliert.",
  },
];

export const WelcomeScreen = ({ onNext }: WelcomeScreenProps) => {
  const [isTransitioning, setIsTransitioning] = useState(false);
  const [isInitializing, setIsInitializing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreateLattice = async () => {
    setError(null);
    setIsInitializing(true);
    try {
      await initRepo();
      setIsTransitioning(true);
      setTimeout(onNext, 800);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Initialisierung fehlgeschlagen.";
      setError(message);
    } finally {
      setIsInitializing(false);
    }
  };

  return (
    <div className={cn(
      "relative flex flex-col items-center justify-center min-h-screen px-5",
      "transition-all duration-1000 ease-out-expo",
      isTransitioning && "scale-95 opacity-0"
    )}>
      <ParticleBackground particleCount={45} />
      
      {/* Lattice expansion animation overlay */}
      {isTransitioning && (
        <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
          <div 
            className="w-4 h-4 rounded-full bg-primary animate-lattice-expand"
            style={{
              boxShadow: "0 0 60px 30px hsl(var(--primary) / 0.5)",
            }}
          />
        </div>
      )}
      
      <div className="relative z-10 flex flex-col items-center max-w-lg">
        {/* Logo */}
        <div className="mb-10">
          <Logo animate />
        </div>
        
        {/* Welcome text */}
        <div 
          className="text-center mb-8 opacity-0 animate-fade-up"
          style={{ animationDelay: "600ms", animationFillMode: "forwards" }}
        >
          <h1 className="text-3xl md:text-4xl font-bold tracking-tighter mb-3 text-foreground">
            Willkommen bei NeuralFS
          </h1>
          <p className="text-base text-muted-foreground max-w-sm mx-auto leading-relaxed">
            Deine Dateien leben nicht mehr in Ordnern.
            <br />
            Sie leben in Bedeutung, Zeit und Beziehungen.
          </p>
        </div>
        
        {/* Philosophy cards */}
        <div className="grid gap-3 w-full mb-8">
          {philosophyCards.map((card, index) => (
            <GlassCard
              key={card.title}
              delay={800 + index * 200}
              className="flex items-start gap-3"
            >
              <div className="flex-shrink-0 p-2.5 rounded-lg bg-primary/10">
                <card.icon className="w-5 h-5 text-primary" />
              </div>
              <div>
                <h3 className="font-semibold text-foreground mb-1">
                  {card.title}
                </h3>
                <p className="text-sm text-muted-foreground">
                  {card.description}
                </p>
              </div>
            </GlassCard>
          ))}
        </div>
        
        {/* CTA Button */}
        <div 
          className="opacity-0 animate-fade-up"
          style={{ animationDelay: "1400ms", animationFillMode: "forwards" }}
        >
          <AnimatedButton onClick={handleCreateLattice} disabled={isInitializing} size="md">
            {isInitializing ? "Initialisiere..." : "Create My Lattice"}
          </AnimatedButton>
        </div>
        {error && (
          <p className="mt-4 text-sm text-warning">{error}</p>
        )}
      </div>
    </div>
  );
};
