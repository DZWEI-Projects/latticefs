import { GlassCard } from "./ui/GlassCard";
import { AnimatedButton } from "./ui/AnimatedButton";
import { Logo } from "./ui/Logo";
import { ParticleBackground } from "./ui/ParticleBackground";
import { Search, Clock, Shield } from "lucide-react";
import { cn } from "@/lib/utils";
import { useState } from "react";

interface WelcomeScreenProps {
  onNext: () => void;
  onInitialize: () => Promise<{ repoPath: string; version: string }>;
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

export const WelcomeScreen = ({ onNext, onInitialize }: WelcomeScreenProps) => {
  const [isTransitioning, setIsTransitioning] = useState(false);
  const [isInitializing, setIsInitializing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreateLattice = async () => {
    setError(null);
    setIsInitializing(true);
    try {
      await onInitialize();
      setIsTransitioning(true);
      // Allow animation to play before transitioning
      setTimeout(onNext, 800);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Lattice konnte nicht initialisiert werden.");
      }
    } finally {
      setIsInitializing(false);
    }
  };

  return (
    <div className={cn(
      "relative flex flex-col items-center justify-center min-h-screen px-6",
      "transition-all duration-1000 ease-out-expo",
      isTransitioning && "scale-95 opacity-0"
    )}>
      <ParticleBackground particleCount={60} />
      
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
      
      <div className="relative z-10 flex flex-col items-center max-w-xl">
        {/* Logo */}
        <div className="mb-16">
          <Logo animate />
        </div>
        
        {/* Welcome text */}
        <div 
          className="text-center mb-12 opacity-0 animate-fade-up"
          style={{ animationDelay: "600ms", animationFillMode: "forwards" }}
        >
          <h1 className="text-4xl md:text-5xl font-bold tracking-tighter mb-4 text-foreground">
            Willkommen bei NeuralFS
          </h1>
          <p className="text-lg text-muted-foreground max-w-md mx-auto leading-relaxed">
            Deine Dateien leben nicht mehr in Ordnern.
            <br />
            Sie leben in Bedeutung, Zeit und Beziehungen.
          </p>
        </div>
        
        {/* Philosophy cards */}
        <div className="grid gap-4 w-full mb-12">
          {philosophyCards.map((card, index) => (
            <GlassCard
              key={card.title}
              delay={800 + index * 200}
              className="flex items-start gap-4"
            >
              <div className="flex-shrink-0 p-3 rounded-xl bg-primary/10">
                <card.icon className="w-6 h-6 text-primary" />
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
          <AnimatedButton onClick={handleCreateLattice} disabled={isInitializing}>
            {isInitializing ? "Initialisiere..." : "Create My Lattice"}
          </AnimatedButton>
          {error && (
            <p className="mt-3 text-sm text-warning text-center">
              {error}
            </p>
          )}
        </div>
      </div>
    </div>
  );
};
