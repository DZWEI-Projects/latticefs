import { useState, useCallback } from "react";
import { cn } from "@/lib/utils";
import { WelcomeScreen } from "./WelcomeScreen";
import { FolderSelection } from "./FolderSelection";
import { NodeGraph } from "./NodeGraph";
import { SecurityCalibration } from "./SecurityCalibration";
import { AhaTutorial } from "./AhaTutorial";

export type OnboardingStage = 1 | 2 | 3 | 4 | 5 | "complete";

interface OnboardingContainerProps {
  onComplete?: () => void;
}

export const OnboardingContainer = ({ onComplete }: OnboardingContainerProps) => {
  const [stage, setStage] = useState<OnboardingStage>(1);
  const [isTransitioning, setIsTransitioning] = useState(false);

  const transitionToStage = useCallback((nextStage: OnboardingStage) => {
    setIsTransitioning(true);
    
    // Short delay for exit animation
    setTimeout(() => {
      setStage(nextStage);
      setIsTransitioning(false);
    }, 100);
  }, []);

  const handleComplete = useCallback(() => {
    transitionToStage("complete");
    onComplete?.();
  }, [onComplete, transitionToStage]);

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
        return (
          <div className="flex items-center justify-center min-h-screen">
            <div className="text-center animate-fade-up">
              <h1 className="text-3xl font-bold tracking-tighter mb-3 text-foreground">
                Willkommen in deinem Lattice
              </h1>
              <p className="text-muted-foreground">
                NeuralFS ist bereit.
              </p>
            </div>
          </div>
        );
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
      {/* Progress indicator */}
      {stage !== "complete" && (
        <div className="fixed top-4 left-1/2 -translate-x-1/2 z-50 flex gap-2">
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
      )}
      
      {renderStage()}
    </div>
  );
};
