import { useState, useEffect } from "react";
import { GlassCard } from "./ui/GlassCard";
import { AnimatedButton } from "./ui/AnimatedButton";
import { ToggleSwitch } from "./ui/ToggleSwitch";
import { ParticleBackground } from "./ui/ParticleBackground";
import { cn } from "@/lib/utils";
import { Shield, Download, History, AlertTriangle, Check } from "lucide-react";
import type { OnboardingSettings } from "@/lib/latticeApi";

interface SecurityCalibrationProps {
  onNext: () => void;
  settings: OnboardingSettings | null;
  onUpdateSettings: (next: OnboardingSettings) => Promise<void>;
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

export const SecurityCalibration = ({ onNext, settings, onUpdateSettings }: SecurityCalibrationProps) => {
  const [localSettings, setLocalSettings] = useState<Record<string, boolean>>(
    Object.fromEntries(securityOptions.map((opt) => [opt.id, opt.defaultEnabled]))
  );
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isAnimatingOut, setIsAnimatingOut] = useState(false);
  const [showShield, setShowShield] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setShowShield(true), 300);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (!settings) return;
    setLocalSettings({
      quarantine: settings.quarantineDownloads,
      versioning: settings.versioning,
      "execute-warning": settings.executeWarning,
    });
  }, [settings]);

  const toggleSetting = (id: string) => {
    setLocalSettings((prev) => ({
      ...prev,
      [id]: !prev[id],
    }));
  };

  const handleProceed = async () => {
    setIsSaving(true);
    setError(null);
    try {
      await onUpdateSettings({
        quarantineDownloads: localSettings.quarantine,
        versioning: localSettings.versioning,
        executeWarning: localSettings["execute-warning"],
      });
      setIsAnimatingOut(true);
      setTimeout(onNext, 800);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Einstellungen konnten nicht gespeichert werden.");
      }
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className={cn(
      "relative flex flex-col items-center justify-center min-h-screen px-6",
      "transition-all duration-1000 ease-out-expo",
      isAnimatingOut && "scale-95 opacity-0"
    )}>
      <ParticleBackground particleCount={30} />
      
      {/* Shield activation overlay */}
      {isAnimatingOut && (
        <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
          <div className="relative">
            <Shield 
              className="w-32 h-32 text-primary animate-scale-in" 
              style={{ filter: "drop-shadow(0 0 40px hsl(var(--primary) / 0.6))" }}
            />
            <Check 
              className="absolute inset-0 m-auto w-16 h-16 text-primary-foreground animate-scale-in"
              style={{ animationDelay: "200ms" }}
            />
          </div>
        </div>
      )}
      
      <div className="relative z-10 flex flex-col items-center max-w-lg w-full">
        {/* Shield icon */}
        <div 
          className={cn(
            "mb-8 transition-all duration-700 ease-out-expo",
            showShield ? "scale-100 opacity-100" : "scale-50 opacity-0"
          )}
        >
          <div className="relative">
            <div 
              className="absolute inset-0 rounded-full blur-2xl animate-pulse-glow"
              style={{ background: "radial-gradient(circle, hsl(var(--primary) / 0.3) 0%, transparent 70%)" }}
            />
            <Shield 
              className="w-20 h-20 text-primary relative z-10"
              style={{ filter: "drop-shadow(0 0 20px hsl(var(--primary) / 0.4))" }}
            />
          </div>
        </div>
        
        {/* Header */}
        <div 
          className="text-center mb-8 opacity-0 animate-fade-up"
          style={{ animationDelay: "200ms", animationFillMode: "forwards" }}
        >
          <h2 className="text-3xl md:text-4xl font-bold tracking-tighter mb-3">
            NeuralFS schützt dich von Grund auf
          </h2>
          <p className="text-muted-foreground">
            Diese Einstellungen sind bereits aktiviert.
          </p>
        </div>
        
        {/* Security options */}
        <div className="w-full space-y-4 mb-6">
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
              <div className="flex items-start gap-4">
                <div className={cn(
                  "flex-shrink-0 p-3 rounded-xl transition-colors duration-300",
                  localSettings[option.id] ? "bg-primary/20" : "bg-muted"
                )}>
                  <option.icon className={cn(
                    "w-5 h-5 transition-colors duration-300",
                    localSettings[option.id] ? "text-primary" : "text-muted-foreground"
                  )} />
                </div>
                
                <div className="flex-grow">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="font-semibold text-foreground">
                      {option.title}
                    </h4>
                    <ToggleSwitch
                      checked={localSettings[option.id]}
                      onChange={() => toggleSetting(option.id)}
                    />
                  </div>
                  <p className={cn(
                    "text-sm transition-all duration-300 overflow-hidden",
                    localSettings[option.id] 
                      ? "text-muted-foreground max-h-20 opacity-100" 
                      : "text-muted-foreground/50 max-h-0 opacity-0"
                  )}>
                    {option.description}
                  </p>
                </div>
              </div>
            </GlassCard>
          ))}
        </div>
        
        {/* Advanced settings link */}
        <p 
          className="text-sm text-muted-foreground/60 mb-8 cursor-pointer hover:text-muted-foreground transition-colors opacity-0 animate-fade-up"
          style={{ animationDelay: "850ms", animationFillMode: "forwards" }}
        >
          Richtlinien später anpassen →
        </p>
        
        {/* CTA Button */}
        <div 
          className="opacity-0 animate-fade-up"
          style={{ animationDelay: "950ms", animationFillMode: "forwards" }}
        >
          <AnimatedButton onClick={handleProceed} disabled={isSaving}>
            {isSaving ? "Speichere..." : "Sieht gut aus"}
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
