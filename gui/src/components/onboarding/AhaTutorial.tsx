import { useState, useEffect, useRef } from "react";
import { cn } from "@/lib/utils";
import { mockFiles, mockViews, mockProjects, getFileById, ViewNode } from "@/data/mockFileSystem";
import { Clock, Folder, Grid, Download, Shield, HelpCircle, Plus, X, Check, Sparkles } from "lucide-react";
import { AnimatedButton } from "./ui/AnimatedButton";
import { GlassCard } from "./ui/GlassCard";

interface AhaTutorialProps {
  onComplete: () => void;
}

const viewIcons: Record<string, typeof Clock> = {
  Clock: Clock,
  Folder: Folder,
  Grid: Grid,
  Download: Download,
  Shield: Shield,
};

const colorClasses: Record<string, { bg: string; border: string; text: string }> = {
  primary: { bg: "bg-primary/20", border: "border-primary/40", text: "text-primary" },
  secondary: { bg: "bg-secondary/20", border: "border-secondary/40", text: "text-secondary" },
  warning: { bg: "bg-warning/20", border: "border-warning/40", text: "text-warning" },
  muted: { bg: "bg-muted", border: "border-muted-foreground/20", text: "text-muted-foreground" },
};

type TutorialStep = "highlight" | "context" | "add-project" | "complete";

export const AhaTutorial = ({ onComplete }: AhaTutorialProps) => {
  const [step, setStep] = useState<TutorialStep>("highlight");
  const [showContextPanel, setShowContextPanel] = useState(false);
  const [selectedProject, setSelectedProject] = useState<string | null>(null);
  const [showConnectionAnimation, setShowConnectionAnimation] = useState(false);
  const [showCelebration, setShowCelebration] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // The highlighted file for the tutorial
  const highlightedFile = mockFiles.find((f) => f.id === "file-3"); // setup_installer.exe
  
  useEffect(() => {
    if (step === "complete") {
      setShowCelebration(true);
      const timer = setTimeout(() => setShowCelebration(false), 2000);
      return () => clearTimeout(timer);
    }
  }, [step]);

  const handleWhyClick = () => {
    setShowContextPanel(true);
    setStep("context");
  };

  const handleAddToProject = (projectId: string) => {
    setSelectedProject(projectId);
    setShowConnectionAnimation(true);
    
    setTimeout(() => {
      setShowConnectionAnimation(false);
      setStep("complete");
    }, 1500);
  };

  const getViewPosition = (index: number, total: number) => {
    const angle = (index / total) * Math.PI * 2 - Math.PI / 2;
    const radius = 160;
    return {
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
    };
  };

  return (
    <div 
      ref={containerRef}
      className="relative flex items-center justify-center min-h-screen overflow-hidden"
    >
      {/* Background */}
      <div className="absolute inset-0 bg-gradient-to-br from-background via-background to-background-deep" />
      
      {/* Celebration particles */}
      {showCelebration && (
        <div className="fixed inset-0 z-50 pointer-events-none overflow-hidden">
          {Array.from({ length: 30 }).map((_, i) => (
            <div
              key={i}
              className="absolute animate-celebration"
              style={{
                left: `${Math.random() * 100}%`,
                top: `${Math.random() * 100}%`,
                width: `${Math.random() * 10 + 5}px`,
                height: `${Math.random() * 10 + 5}px`,
                background: `hsl(${Math.random() * 60 + 200}, 70%, 60%)`,
                borderRadius: Math.random() > 0.5 ? "50%" : "2px",
                animationDelay: `${Math.random() * 500}ms`,
              }}
            />
          ))}
        </div>
      )}
      
      {/* SVG for connections */}
      <svg className="absolute inset-0 w-full h-full pointer-events-none" style={{ zIndex: 1 }}>
        <defs>
          <linearGradient id="tutorial-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="hsl(var(--primary))" stopOpacity="0.6" />
            <stop offset="100%" stopColor="hsl(var(--secondary))" stopOpacity="0.6" />
          </linearGradient>
        </defs>
        
        <g transform={`translate(${containerRef.current?.clientWidth ? containerRef.current.clientWidth / 2 - 150 : 350}, ${containerRef.current?.clientHeight ? containerRef.current.clientHeight / 2 : 400})`}>
          {/* Existing connections */}
          {mockViews.map((view, index) => {
            const pos = getViewPosition(index, mockViews.length);
            return (
              <line
                key={`hub-${view.id}`}
                x1="0"
                y1="0"
                x2={pos.x}
                y2={pos.y}
                stroke="url(#tutorial-gradient)"
                strokeWidth="1"
                opacity="0.3"
              />
            );
          })}
          
          {/* New connection animation */}
          {showConnectionAnimation && selectedProject && (
            <line
              x1={getViewPosition(4, mockViews.length).x - 30}
              y1={getViewPosition(4, mockViews.length).y}
              x2={getViewPosition(1, mockViews.length).x}
              y2={getViewPosition(1, mockViews.length).y}
              stroke="hsl(var(--secondary))"
              strokeWidth="2"
              strokeDasharray="1000"
              className="animate-draw-line"
            />
          )}
        </g>
      </svg>
      
      {/* Node graph (simplified) */}
      <div 
        className="relative z-10"
        style={{ width: 400, height: 400, marginRight: showContextPanel ? "300px" : "0" }}
      >
        {/* Central hub */}
        <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-16 h-16 rounded-full bg-primary/20 border-2 border-primary/40 flex items-center justify-center">
          <div className="w-8 h-8 rounded-full bg-primary/30 flex items-center justify-center">
            <div className="w-4 h-4 rounded-full bg-primary" />
          </div>
        </div>
        
        {/* View nodes */}
        {mockViews.map((view, index) => {
          const pos = getViewPosition(index, mockViews.length);
          const Icon = viewIcons[view.icon] || Folder;
          const colors = colorClasses[view.color];
          const isProjectView = view.id === "projekte";
          
          return (
            <div
              key={view.id}
              className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transition-all duration-500"
              style={{
                transform: `translate(calc(-50% + ${pos.x}px), calc(-50% + ${pos.y}px))`,
              }}
            >
              <div className={cn(
                "w-12 h-12 rounded-xl border-2 flex items-center justify-center transition-all duration-300",
                colors.bg,
                colors.border,
                isProjectView && showConnectionAnimation && "glow-secondary scale-110"
              )}>
                <Icon className={cn("w-5 h-5", colors.text)} />
              </div>
              <span className="absolute -bottom-5 left-1/2 -translate-x-1/2 text-xs font-medium text-foreground whitespace-nowrap">
                {view.name}
              </span>
            </div>
          );
        })}
        
        {/* Highlighted file node */}
        {highlightedFile && (
          <div
            className={cn(
              "absolute left-1/2 top-1/2 transition-all duration-500 cursor-pointer group",
              step === "highlight" && "animate-node-pulse"
            )}
            style={{
              transform: `translate(calc(-50% + ${getViewPosition(4, mockViews.length).x - 30}px), calc(-50% + ${getViewPosition(4, mockViews.length).y}px))`,
            }}
            onClick={handleWhyClick}
          >
            <div className={cn(
              "w-10 h-10 rounded-lg bg-warning/20 border-2 border-warning/40",
              "flex items-center justify-center text-xs font-medium text-warning",
              "transition-all duration-300",
              step === "highlight" && "ring-2 ring-warning/50 ring-offset-2 ring-offset-background"
            )}>
              EXE
            </div>
            
            {/* "Why is this here?" prompt */}
            {step === "highlight" && (
              <div className="absolute top-full left-1/2 -translate-x-1/2 mt-3 glass-strong rounded-lg px-3 py-2 animate-fade-up whitespace-nowrap">
                <div className="flex items-center gap-2 text-sm">
                  <HelpCircle className="w-4 h-4 text-primary" />
                  <span className="text-foreground">Warum ist das hier?</span>
                </div>
                <div className="absolute -top-2 left-1/2 -translate-x-1/2 w-0 h-0 border-l-8 border-r-8 border-b-8 border-transparent border-b-glass-border" />
              </div>
            )}
          </div>
        )}
      </div>
      
      {/* Context panel */}
      {showContextPanel && highlightedFile && (
        <div 
          className={cn(
            "absolute right-8 top-1/2 -translate-y-1/2 w-80 z-20",
            "animate-slide-in-right"
          )}
        >
          <GlassCard hover={false} delay={0} className="opacity-100">
            <button
              onClick={() => {
                setShowContextPanel(false);
                setStep("highlight");
              }}
              className="absolute top-4 right-4 text-muted-foreground hover:text-foreground transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
            
            <div className="mb-4">
              <h3 className="font-semibold text-foreground mb-1">
                {highlightedFile.name}
              </h3>
              <p className="text-sm text-muted-foreground">
                Dieses Objekt erscheint hier, weil:
              </p>
            </div>
            
            <div className="space-y-3 mb-6">
              <div className="flex items-center gap-3 text-sm">
                <Clock className="w-4 h-4 text-primary" />
                <span className="text-muted-foreground">Heute heruntergeladen</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Download className="w-4 h-4 text-warning" />
                <span className="text-muted-foreground">
                  Tag: <span className="text-warning">inbox:downloads</span>
                </span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Shield className="w-4 h-4 text-warning" />
                <span className="text-muted-foreground">In Quarantäne</span>
              </div>
            </div>
            
            {step === "context" && (
              <>
                <div className="border-t border-border pt-4 mb-4">
                  <p className="text-sm text-muted-foreground mb-3">
                    Zu einem Projekt hinzufügen?
                  </p>
                  <div className="space-y-2">
                    {mockProjects.map((project) => (
                      <button
                        key={project.id}
                        onClick={() => handleAddToProject(project.id)}
                        className={cn(
                          "w-full flex items-center gap-3 p-2 rounded-lg",
                          "bg-muted/50 hover:bg-muted transition-colors",
                          "text-left text-sm"
                        )}
                      >
                        <div 
                          className="w-3 h-3 rounded-full"
                          style={{ backgroundColor: project.color }}
                        />
                        <span className="text-foreground">{project.name}</span>
                        <Plus className="w-4 h-4 text-muted-foreground ml-auto" />
                      </button>
                    ))}
                  </div>
                </div>
              </>
            )}
            
            {step === "add-project" && selectedProject && (
              <div className="flex items-center gap-2 text-sm text-secondary">
                <Check className="w-4 h-4" />
                <span>Hinzugefügt zu {mockProjects.find((p) => p.id === selectedProject)?.name}</span>
              </div>
            )}
          </GlassCard>
        </div>
      )}
      
      {/* Completion message */}
      {step === "complete" && (
        <div className="absolute bottom-12 left-1/2 -translate-x-1/2 z-30 text-center animate-fade-up">
          <div className="glass-strong rounded-2xl p-8 max-w-md">
            <Sparkles className="w-10 h-10 text-primary mx-auto mb-4" />
            <h3 className="text-2xl font-bold tracking-tight mb-3">
              Das ist alles.
            </h3>
            <p className="text-muted-foreground mb-6">
              Du weißt bereits genug, um mit NeuralFS zu arbeiten.
            </p>
            <AnimatedButton onClick={onComplete}>
              Los geht's
            </AnimatedButton>
          </div>
        </div>
      )}
    </div>
  );
};
