import { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cn } from "@/lib/utils";
import { WelcomeScreen } from "./WelcomeScreen";
import { FolderSelection } from "./FolderSelection";
import { NodeGraph } from "./NodeGraph";
import { SecurityCalibration } from "./SecurityCalibration";
import { AhaTutorial } from "./AhaTutorial";
import { CompleteScreen } from "./CompleteScreen";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export type OnboardingStage = 1 | 2 | 3 | 4 | 5 | "complete";

const COMPLETE_REST_BEFORE_EXIT_MS = 6500;
const COMPLETE_EXIT_DURATION_MS = 2200;

interface OnboardingContainerProps {
  onComplete?: () => void;
}

export const OnboardingContainer = ({ onComplete }: OnboardingContainerProps) => {
  const navigate = useNavigate();
  const [stage, setStage] = useState<OnboardingStage>(1);
  const [isTransitioning, setIsTransitioning] = useState(false);
  const [isExiting, setIsExiting] = useState(false);
  const [devStepInput, setDevStepInput] = useState("");

  const transitionToStage = useCallback((nextStage: OnboardingStage) => {
    setIsTransitioning(true);
    
    // Short delay for exit animation
    setTimeout(() => {
      setStage(nextStage);
      setIsTransitioning(false);
    }, 100);
  }, []);

  const handleComplete = useCallback(() => {
    localStorage.setItem("lfs-onboarding-complete", "true");
    transitionToStage("complete");
    onComplete?.();
  }, [onComplete, transitionToStage]);

  // Trigger exit animation after text has rested
  useEffect(() => {
    if (stage === "complete" && !isExiting) {
      const timer = setTimeout(() => {
        setIsExiting(true);
      }, COMPLETE_REST_BEFORE_EXIT_MS);
      return () => clearTimeout(timer);
    }
  }, [stage, isExiting]);

  // Navigate after exit animation completes
  useEffect(() => {
    if (isExiting) {
      const timer = setTimeout(() => {
        navigate("/nexus");
      }, COMPLETE_EXIT_DURATION_MS);
      return () => clearTimeout(timer);
    }
  }, [isExiting, navigate]);

  const handleDevStepJump = useCallback(() => {
    const stepNum = parseInt(devStepInput, 10);
    if (stepNum >= 1 && stepNum <= 5) {
      transitionToStage(stepNum as OnboardingStage);
      setDevStepInput("");
    }
  }, [devStepInput, transitionToStage]);

  const handleDragStart = useCallback((e: React.MouseEvent) => {
    if (e.buttons === 1) {
      // Primary (left) button
      getCurrentWindow().startDragging();
    }
  }, []);

  const renderStage = () => {
    switch (stage) {
      case 1:
        return <WelcomeScreen onNext={() => transitionToStage(2)} />;
      case 2:
        return <FolderSelection onNext={() => transitionToStage(3)} />;
      case 3:
        return <NodeGraph onNext={() => transitionToStage(4)} showTutorial />;
      case 4:
        return <SecurityCalibration onNext={() => transitionToStage(5)} />;
      case 5:
        return <AhaTutorial onComplete={handleComplete} />;
      case "complete":
        return <CompleteScreen isExiting={isExiting} />;
      default:
        return null;
    }
  };

  return (
    <div 
      className={cn(
        "min-h-screen w-full overflow-hidden",
        "transition-opacity duration-300 ease-out-expo",
        isTransitioning && "opacity-0"
      )}
    >
      {/* Drag region / Progress indicator */}
      {stage !== "complete" && (
        <div 
          className="fixed top-0 left-0 right-0 z-50 h-12 flex items-center justify-center px-4 select-none cursor-default"
          onMouseDown={handleDragStart}
        >
          <div className="flex items-center gap-2 pointer-events-none">
            
            {[1, 2, 3, 4, 5].map((s) => (
              <div
                key={s}
                className={cn(
                  "w-1.5 h-1.5 rounded-full transition-all duration-400",
                  s === stage
                    ? "w-6 bg-primary"
                    : s < (stage as number)
                    ? "bg-primary/60"
                    : "bg-muted-foreground/30"
                )}
              />
            ))}
          </div>
          
          {/* Dev-only step navigator */}
          {import.meta.env.DEV && (
            <div 
              className="absolute right-4 flex items-center gap-2 pointer-events-auto"
              onMouseDown={(e) => e.stopPropagation()}
            >
              <Input
                type="number"
                min="1"
                max="5"
                placeholder="Schritt"
                value={devStepInput}
                onChange={(e) => setDevStepInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleDevStepJump();
                  }
                }}
                className="w-16 h-7 text-xs"
              />
              <Button
                size="sm"
                variant="outline"
                onClick={handleDevStepJump}
                className="h-7 text-xs"
              >
                Los
              </Button>
            </div>
          )}
        </div>
      )}
      
      {renderStage()}
    </div>
  );
};
