import { useState, useEffect, useCallback, useRef } from "react";
import { cn } from "@/lib/utils";
import { mockViews, ViewNode } from "@/data/mockFileSystem";
import { Clock, Folder, Grid, Download, Shield, X } from "lucide-react";
import { AnimatedButton } from "./ui/AnimatedButton";
import { getOnboardingGraph, type OnboardingFile } from "@/lib/lfs";

interface NodeGraphProps {
  onNext: () => void;
  showTutorial?: boolean;
}

// Node container dimensions - used for both the container and SVG positioning
const NODE_CONTAINER_SIZE = 440;
const NODE_CENTER = NODE_CONTAINER_SIZE / 2; // 220

const viewIcons: Record<string, typeof Clock> = {
  Clock: Clock,
  Folder: Folder,
  Grid: Grid,
  Download: Download,
  Shield: Shield,
};

const colorClasses: Record<string, { bg: string; border: string; text: string }> = {
  primary: {
    bg: "bg-primary/20",
    border: "border-primary/40",
    text: "text-primary",
  },
  secondary: {
    bg: "bg-secondary/20",
    border: "border-secondary/40",
    text: "text-secondary",
  },
  warning: {
    bg: "bg-warning/20",
    border: "border-warning/40",
    text: "text-warning",
  },
  muted: {
    bg: "bg-muted",
    border: "border-muted-foreground/20",
    text: "text-muted-foreground",
  },
};

interface TooltipData {
  view: ViewNode;
  position: { x: number; y: number };
}

export const NodeGraph = ({ onNext, showTutorial = true }: NodeGraphProps) => {
  const [animationPhase, setAnimationPhase] = useState(0);
  const [hoveredView, setHoveredView] = useState<string | null>(null);
  const [hoveredFile, setHoveredFile] = useState<string | null>(null);
  const [activeTooltip, setActiveTooltip] = useState<TooltipData | null>(null);
  const [tooltipIndex, setTooltipIndex] = useState(0);
  const [showInsight, setShowInsight] = useState(false);
  const [files, setFiles] = useState<OnboardingFile[]>([]);
  const [isLoadingFiles, setIsLoadingFiles] = useState(true);
  const containerRef = useRef<HTMLDivElement>(null);

  // Animation phases
  useEffect(() => {
    const timers = [
      setTimeout(() => setAnimationPhase(1), 300),   // Hub appears
      setTimeout(() => setAnimationPhase(2), 800),   // View nodes appear
      setTimeout(() => setAnimationPhase(3), 1500),  // Connections draw
      setTimeout(() => setAnimationPhase(4), 2200),  // File nodes appear
      setTimeout(() => setAnimationPhase(5), 3000),  // Tooltips begin
    ];
    return () => timers.forEach(clearTimeout);
  }, []);

  useEffect(() => {
    let isMounted = true;
    setIsLoadingFiles(true);
    getOnboardingGraph()
      .then((data) => {
        if (!isMounted) return;
        setFiles(data.files);
      })
      .catch(() => {
        if (!isMounted) return;
        setFiles([]);
      })
      .finally(() => {
        if (!isMounted) return;
        setIsLoadingFiles(false);
      });
    return () => {
      isMounted = false;
    };
  }, []);

  // Show tooltips sequentially
  useEffect(() => {
    if (animationPhase < 5 || !showTutorial) return;
    
    const showNextTooltip = () => {
      if (tooltipIndex < mockViews.length) {
        const view = mockViews[tooltipIndex];
        // Position tooltip near the view node
        const angle = (tooltipIndex / mockViews.length) * Math.PI * 2 - Math.PI / 2;
        const radius = 150;
        setActiveTooltip({
          view,
          position: {
            x: Math.cos(angle) * radius,
            y: Math.sin(angle) * radius,
          },
        });
      } else if (tooltipIndex === mockViews.length) {
        setActiveTooltip(null);
        setShowInsight(true);
      }
    };

    showNextTooltip();
  }, [animationPhase, tooltipIndex, showTutorial]);

  const advanceTooltip = useCallback(() => {
    setTooltipIndex((prev) => prev + 1);
  }, []);

  // Calculate node positions in a circular layout
  const getViewPosition = (index: number, total: number) => {
    const angle = (index / total) * Math.PI * 2 - Math.PI / 2;
    const radius = 150;
    return {
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
    };
  };

  // Get file positions clustered around their primary view
  const getFilePosition = (file: OnboardingFile, fileIndex: number) => {
    const primaryView = file.views[0];
    const viewIndex = mockViews.findIndex((v) => v.id === primaryView);
    if (viewIndex === -1) return { x: 0, y: 0 };
    
    const viewPos = getViewPosition(viewIndex, mockViews.length);
    const offset = 42 + fileIndex * 4;
    const angle = (fileIndex * 0.8) + viewIndex;
    
    return {
      x: viewPos.x + Math.cos(angle) * offset,
      y: viewPos.y + Math.sin(angle) * offset,
    };
  };

  // Get files that appear in multiple views (for highlighting)
  const multiViewFiles = files.filter((f) => f.views.length > 1);

  return (
    <div 
      ref={containerRef}
      className="relative flex items-center justify-center min-h-screen overflow-hidden"
    >
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-background via-background to-background-deep" />
      
      {/* Nodes container */}
      <div 
        className="relative z-10"
        style={{ width: NODE_CONTAINER_SIZE, height: NODE_CONTAINER_SIZE }}
      >
        {/* SVG for connections - positioned inside the node container for correct alignment */}
        <svg 
          className="absolute inset-0 w-full h-full pointer-events-none overflow-visible"
          style={{ zIndex: 0 }}
        >
          <defs>
            <linearGradient id="connection-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="hsl(var(--primary))" stopOpacity="0.4" />
              <stop offset="100%" stopColor="hsl(var(--secondary))" stopOpacity="0.4" />
            </linearGradient>
          </defs>
          
          <g transform={`translate(${NODE_CENTER}, ${NODE_CENTER})`}>
            {/* Connections from hub to views */}
            {animationPhase >= 3 && mockViews.map((view, index) => {
              const pos = getViewPosition(index, mockViews.length);
              return (
                <line
                  key={`hub-${view.id}`}
                  x1="0"
                  y1="0"
                  x2={pos.x}
                  y2={pos.y}
                  stroke="url(#connection-gradient)"
                  strokeWidth="1"
                  className={cn(
                    "transition-opacity duration-500",
                    hoveredView === view.id ? "opacity-100" : "opacity-40"
                  )}
                  strokeDasharray="1000"
                  strokeDashoffset="0"
                  style={{
                    animation: "draw-line ease-out forwards",
                    animationDuration: "2s",
                    animationDelay: `${index * 100}ms`,
                  }}
                />
              );
            })}
            
            {/* Connections between files and their views */}
            {animationPhase >= 4 && hoveredFile && (
              <>
                {(() => {
                  const file = files.find((f) => f.id === hoveredFile);
                  if (!file) return null;
                  const filePos = getFilePosition(file, files.indexOf(file));
                  
                  return file.views.map((viewId) => {
                    const viewIndex = mockViews.findIndex((v) => v.id === viewId);
                    if (viewIndex === -1) return null;
                    const viewPos = getViewPosition(viewIndex, mockViews.length);
                    
                    return (
                      <line
                        key={`file-${file.id}-${viewId}`}
                        x1={filePos.x}
                        y1={filePos.y}
                        x2={viewPos.x}
                        y2={viewPos.y}
                        stroke="hsl(var(--primary))"
                        strokeWidth="1.5"
                        strokeOpacity="0.8"
                        className="animate-draw-line"
                      />
                    );
                  });
                })()}
              </>
            )}
          </g>
        </svg>
        {/* Central hub */}
        <div 
          className={cn(
            "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
            "w-16 h-16 rounded-full bg-primary/20 border-2 border-primary/40",
            "flex items-center justify-center",
            "transition-all duration-700 ease-out-expo",
            animationPhase >= 1 ? "scale-100 opacity-100" : "scale-0 opacity-0"
          )}
        >
          <div className="w-10 h-10 rounded-full bg-primary/30 flex items-center justify-center animate-pulse-glow">
            <div className="w-5 h-5 rounded-full bg-primary" />
          </div>
          <span className="absolute -bottom-6 text-xs font-medium text-foreground whitespace-nowrap">
            Dein Lattice
          </span>
        </div>
        
        {/* View nodes */}
        {mockViews.map((view, index) => {
          const pos = getViewPosition(index, mockViews.length);
          const Icon = viewIcons[view.icon] || Folder;
          const colors = colorClasses[view.color];
          
          return (
            <div
              key={view.id}
              className={cn(
                "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
                "transition-all duration-700 ease-out-expo cursor-pointer",
                animationPhase >= 2 ? "scale-100 opacity-100" : "scale-0 opacity-0",
                hoveredView === view.id && "scale-110"
              )}
              style={{
                transform: `translate(calc(-50% + ${pos.x}px), calc(-50% + ${pos.y}px))`,
                transitionDelay: `${index * 80}ms`,
              }}
              onMouseEnter={() => setHoveredView(view.id)}
              onMouseLeave={() => setHoveredView(null)}
            >
              <div className={cn(
                "w-12 h-12 rounded-lg border-2 flex items-center justify-center",
                "transition-all duration-300",
                colors.bg,
                colors.border,
                hoveredView === view.id && "glow-primary"
              )}>
                <Icon className={cn("w-5 h-5", colors.text)} />
              </div>
              <span className="absolute -bottom-5 left-1/2 -translate-x-1/2 text-[11px] font-medium text-foreground whitespace-nowrap">
                {view.name}
              </span>
            </div>
          );
        })}
        
        {/* File nodes (shown only on hover or in animation phase 4+) */}
        {animationPhase >= 4 && !isLoadingFiles && multiViewFiles.slice(0, 6).map((file, index) => {
          const pos = getFilePosition(file, index);
          const isHighlighted = hoveredFile === file.id || (hoveredView && file.views.includes(hoveredView));
          
          return (
            <div
              key={file.id}
              className={cn(
                "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
                "transition-all duration-500 ease-out-expo cursor-pointer",
                isHighlighted ? "scale-125 z-20" : "scale-100 z-10"
              )}
              style={{
                transform: `translate(calc(-50% + ${pos.x}px), calc(-50% + ${pos.y}px))`,
                opacity: isHighlighted ? 1 : 0.6,
              }}
              onMouseEnter={() => setHoveredFile(file.id)}
              onMouseLeave={() => setHoveredFile(null)}
            >
              <div className={cn(
                "w-7 h-7 rounded-md bg-muted border border-muted-foreground/20",
                "flex items-center justify-center text-[10px] font-medium",
                "transition-all duration-300",
                isHighlighted && "border-primary/60 bg-primary/10"
              )}>
                {file.extension?.toUpperCase().slice(0, 3)}
              </div>
              {isHighlighted && (
                <span className="absolute -bottom-4 left-1/2 -translate-x-1/2 text-[9px] text-foreground whitespace-nowrap max-w-[92px] truncate">
                  {file.name}
                </span>
              )}
            </div>
          );
        })}
      </div>
      
      {/* Tooltip overlay */}
      {activeTooltip && showTutorial && (
        <div 
          className="absolute z-30 transition-transform duration-500 ease-out-expo will-change-transform"
          style={{
            left: "50%",
            top: "50%",
            transform: `translate(calc(${activeTooltip.position.x + 70}px), calc(${activeTooltip.position.y - 16}px))`,
          }}
        >
          <div className="glass-strong rounded-lg p-3 max-w-[220px] animate-scale-in">
            <div className="flex items-start gap-2.5">
              <div className={cn(
                "p-1.5 rounded-md flex-shrink-0",
                colorClasses[activeTooltip.view.color].bg
              )}>
                {(() => {
                  const Icon = viewIcons[activeTooltip.view.icon] || Folder;
                  return <Icon className={cn("w-4 h-4", colorClasses[activeTooltip.view.color].text)} />;
                })()}
              </div>
              <div>
                <h4 className="font-semibold text-foreground mb-1">
                  {activeTooltip.view.name}
                </h4>
                <p className="text-xs text-muted-foreground">
                  {activeTooltip.view.description}
                </p>
              </div>
            </div>
            <button
              onClick={advanceTooltip}
              className="mt-2 text-xs text-primary hover:text-primary/80 transition-colors"
            >
              Weiter →
            </button>
          </div>
        </div>
      )}
      
      {/* Insight card */}
      {showInsight && showTutorial && (
        <div className="absolute bottom-6 left-1/2 -translate-x-1/2 z-30 glass-strong rounded-xl p-5 max-w-sm animate-slide-in-up">
          <button
            onClick={() => setShowInsight(false)}
            className="absolute top-2.5 right-2.5 text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
          
          <p className="text-sm text-foreground mb-4 leading-relaxed">
            <span className="text-primary font-medium">Diese Datei erscheint an mehreren Stellen</span> — aber sie existiert nur einmal. Das sind keine Ordner. Das sind <span className="text-secondary">Perspektiven</span>.
          </p>
          
          <AnimatedButton onClick={onNext} size="sm">
            Weiter
          </AnimatedButton>
        </div>
      )}
      
      {/* Skip button if not showing tutorial */}
      {!showTutorial && animationPhase >= 4 && (
        <div className="absolute bottom-6 left-1/2 -translate-x-1/2 z-30">
          <AnimatedButton onClick={onNext} size="sm">
            Weiter
          </AnimatedButton>
        </div>
      )}
    </div>
  );
};
