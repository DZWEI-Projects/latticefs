import { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { motion } from "motion/react";
import { cn } from "@/lib/utils";
import { WelcomeScreen } from "./WelcomeScreen";
import { FolderSelection } from "./FolderSelection";
import { NodeGraph } from "./NodeGraph";
import { SecurityCalibration } from "./SecurityCalibration";
import { AhaTutorial } from "./AhaTutorial";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export type OnboardingStage = 1 | 2 | 3 | 4 | 5 | "complete";

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
    transitionToStage("complete");
    onComplete?.();
  }, [onComplete, transitionToStage]);

  // Trigger exit animation after text has rested
  useEffect(() => {
    if (stage === "complete" && !isExiting) {
      const timer = setTimeout(() => {
        setIsExiting(true);
      }, 5500);
      return () => clearTimeout(timer);
    }
  }, [stage, isExiting]);

  // Navigate after exit animation completes
  useEffect(() => {
    if (isExiting) {
      // Wait a bit to not have it jump too hard
      const timer = setTimeout(() => {
        navigate("/nexus");
      }, 1995);
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
        return (
          <motion.div 
            className="flex items-center justify-center min-h-screen"
            initial={{ opacity: 0 }}
            animate={isExiting ? { opacity: 0 } : { opacity: 1 }}
            transition={{ 
              duration: 2.0, 
              delay: isExiting ? 0 : 1.0, 
              ease: [0.16, 1, 0.3, 1] 
            }}
          >
            <motion.div 
              className="text-center"
              initial="hidden"
              animate={isExiting ? "exit" : "visible"}
              variants={{
                visible: {
                  transition: {
                    staggerChildren: 0.6,
                    delayChildren: 1.0,
                  },
                },
                exit: {
                  transition: {
                    staggerChildren: 0.15,
                  },
                },
              }}
            >
              <motion.h1 
                className="text-3xl font-bold tracking-tighter mb-3 text-foreground"
                variants={{
                  hidden: { 
                    opacity: 0, 
                    y: 30,
                    scale: 0.97,
                    filter: "blur(12px)",
                    rotateX: 10,
                  },
                  visible: { 
                    opacity: 1, 
                    y: 0,
                    scale: 1,
                    filter: "blur(0px)",
                    rotateX: 0,
                    transition: {
                      type: "spring",
                      damping: 25,
                      stiffness: 40,
                      mass: 1.4,
                      restDelta: 0.001,
                    }
                  },
                  exit: {
                    opacity: 0,
                    y: -25,
                    scale: 0.98,
                    filter: "blur(8px)",
                    rotateX: -5,
                    transition: {
                      type: "spring",
                      damping: 35,
                      stiffness: 100,
                    }
                  },
                }}
                style={{ perspective: 800 }}
              >
                Willkommen in deinem Lattice
              </motion.h1>
              <motion.p 
                className="text-muted-foreground"
                variants={{
                  hidden: { 
                    opacity: 0, 
                    y: 20,
                    filter: "blur(8px)",
                  },
                  visible: { 
                    opacity: 1, 
                    y: 0,
                    filter: "blur(0px)",
                    transition: {
                      type: "spring",
                      damping: 28,
                      stiffness: 45,
                      mass: 1.2,
                      restDelta: 0.001,
                    }
                  },
                  exit: {
                    opacity: 0,
                    y: -18,
                    filter: "blur(6px)",
                    transition: {
                      type: "spring",
                      damping: 35,
                      stiffness: 120,
                    }
                  },
                }}
              >
                NeuralFS ist bereit.
              </motion.p>
            </motion.div>
          </motion.div>
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
