import { useState, useEffect } from "react";
import { GlassCard } from "./ui/GlassCard";
import { AnimatedButton } from "./ui/AnimatedButton";
import { ToggleSwitch } from "./ui/ToggleSwitch";
import { ParticleBackground } from "./ui/ParticleBackground";
import { cn } from "@/lib/utils";
import { Shield, Download, History, AlertTriangle, Check } from "lucide-react";

interface SecurityCalibrationProps {
  onNext: () => void;
}

interface SecurityOption {
  id: string;
  icon: typeof Shield;
  title: string;
  description: string;
  defaultEnabled: boolean;
}

const securityOptions: SecurityOption[] = [
  {
    id: "quarantine",
    icon: Download,
    title: "Quarantäne für neue Downloads",
    description: "Dateien, die aus dem Internet geladen werden, werden automatisch in Quarantäne gelegt, bis du sie explizit freigibst.",
    defaultEnabled: true,
  },
  {
    id: "versioning",
    icon: History,
    title: "Versionierung",
    description: "Jedes Objekt wird versioniert, sodass du niemals eine Änderung verlierst — spring einfach zurück.",
    defaultEnabled: true,
  },
  {
    id: "execute-warning",
    icon: AlertTriangle,
    title: "Frage vor dem Ausführen unbekannter Objekte",
    description: "Du wirst vor der Ausführung oder dem Öffnen unbekannter Objekte gewarnt.",
    defaultEnabled: true,
  },
];

export const SecurityCalibration = ({ onNext }: SecurityCalibrationProps) => {
  const [settings, setSettings] = useState<Record<string, boolean>>(
    Object.fromEntries(securityOptions.map((opt) => [opt.id, opt.defaultEnabled]))
  );
  const [isAnimatingOut, setIsAnimatingOut] = useState(false);
  const [showShield, setShowShield] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setShowShield(true), 300);
    return () => clearTimeout(timer);
  }, []);

  const toggleSetting = (id: string) => {
    setSettings((prev) => ({
      ...prev,
      [id]: !prev[id],
    }));
  };

  const handleProceed = () => {
    setIsAnimatingOut(true);
    setTimeout(onNext, 800);
  };

  return (
    <div className={cn(
      "relative flex flex-col items-center justify-center min-h-screen px-5",
      "transition-all duration-1000 ease-out-expo",
      isAnimatingOut && "scale-95 opacity-0"
    )}>
      <ParticleBackground particleCount={28} />
      
      {/* Shield activation overlay */}
      {isAnimatingOut && (
          <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
          <div className="relative">
            <Shield 
              className="w-24 h-24 text-primary animate-scale-in" 
              style={{ filter: "drop-shadow(0 0 40px hsl(var(--primary) / 0.6))" }}
            />
            <Check 
              className="absolute inset-0 m-auto w-12 h-12 text-primary-foreground animate-scale-in"
              style={{ animationDelay: "200ms" }}
            />
          </div>
        </div>
      )}
      
      <div className="relative z-10 flex flex-col items-center max-w-md w-full">
        {/* Shield icon */}
        <div 
          className={cn(
            "mb-6 transition-all duration-700 ease-out-expo",
            showShield ? "scale-100 opacity-100" : "scale-50 opacity-0"
          )}
        >
          <div className="relative">
            <div 
              className="absolute inset-0 rounded-full blur-xl animate-pulse-glow"
              style={{ background: "radial-gradient(circle, hsl(var(--primary) / 0.3) 0%, transparent 70%)" }}
            />
            <Shield 
              className="w-16 h-16 text-primary relative z-10"
              style={{ filter: "drop-shadow(0 0 20px hsl(var(--primary) / 0.4))" }}
            />
          </div>
        </div>
        
        {/* Header */}
        <div 
          className="text-center mb-6 opacity-0 animate-fade-up"
          style={{ animationDelay: "200ms", animationFillMode: "forwards" }}
        >
          <h2 className="text-2xl md:text-3xl font-bold tracking-tighter mb-2">
            NeuralFS schützt dich von Grund auf
          </h2>
          <p className="text-sm text-muted-foreground">
            Diese Einstellungen sind bereits aktiviert.
          </p>
        </div>
        
        {/* Security options */}
        <div className="w-full space-y-3 mb-5">
          {securityOptions.map((option, index) => (
            <GlassCard
              key={option.id}
              delay={400 + index * 150}
              hover={false}
              className={cn(
                "transition-all duration-300",
                settings[option.id] && "border-primary/30"
              )}
            >
              <div className="flex items-start gap-3">
                <div className={cn(
                  "flex-shrink-0 rounded-lg transition-all duration-300",
                  settings[option.id] ? "bg-primary/20" : "bg-muted",
                  settings[option.id] ? "p-2" : "p-1",
                )}>
                  <option.icon className={cn(
                    "transition-all duration-300",
                    settings[option.id] ? "text-primary" : "text-muted-foreground",
                    settings[option.id] ? "w-4 h-4" : "w-3 h-3"
                  )} />
                </div>
                
                <div className="flex-grow">
                  <div className={cn(
                    "flex items-center justify-between",
                    settings[option.id] ? "mb-1.5" : "mb-0"
                  )}>
                    <h4 className="font-semibold text-foreground">
                      {option.title}
                    </h4>
                    <ToggleSwitch
                      checked={settings[option.id]}
                      onChange={() => toggleSetting(option.id)}
                    />
                  </div>
                  <div
                    className={cn(
                      "grid overflow-hidden transition-all duration-300",
                      settings[option.id]
                        ? "grid-rows-[1fr] opacity-100"
                        : "grid-rows-[0fr] opacity-0"
                    )}
                  >
                    <p className="text-sm text-muted-foreground overflow-hidden">
                      {option.description}
                    </p>
                  </div>
                </div>
              </div>
            </GlassCard>
          ))}
        </div>
        
        {/* Advanced settings link */}
        <p 
          className="text-xs text-muted-foreground/60 mb-6 cursor-pointer hover:text-muted-foreground transition-colors opacity-0 animate-fade-up"
          style={{ animationDelay: "850ms", animationFillMode: "forwards" }}
        >
          Richtlinien später anpassen →
        </p>
        
        {/* CTA Button */}
        <div 
          className="opacity-0 animate-fade-up"
          style={{ animationDelay: "950ms", animationFillMode: "forwards" }}
        >
          <AnimatedButton onClick={handleProceed} size="md">
            Sieht gut aus
          </AnimatedButton>
        </div>
      </div>
    </div>
  );
};
