import { useState, useCallback, useEffect } from "react";
import { cn } from "@/lib/utils";
import { WelcomeScreen } from "./WelcomeScreen";
import { FolderSelection } from "./FolderSelection";
import { NodeGraph } from "./NodeGraph";
import { SecurityCalibration } from "./SecurityCalibration";
import { AhaTutorial } from "./AhaTutorial";
import {
  fetchFolderOptions,
  importFolders,
  initRepo,
  seedDemoFiles,
  type FolderOption,
} from "@/lib/latticeApi";
import { useOnboardingData } from "@/hooks/use-onboarding-data";

export type OnboardingStage = 1 | 2 | 3 | 4 | 5 | "complete";

interface OnboardingContainerProps {
  onComplete?: () => void;
}

export const OnboardingContainer = ({ onComplete }: OnboardingContainerProps) => {
  const [stage, setStage] = useState<OnboardingStage>(1);
  const [isTransitioning, setIsTransitioning] = useState(false);
  const [folderOptions, setFolderOptions] = useState<FolderOption[]>([]);
  const [folderError, setFolderError] = useState<string | null>(null);
  const { data, refresh, assignToProject, settings, updateSecuritySettings } = useOnboardingData();

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

  useEffect(() => {
    const loadFolders = async () => {
      try {
        const options = await fetchFolderOptions();
        setFolderOptions(options);
      } catch (err) {
        if (err instanceof Error) {
          setFolderError(err.message);
        } else {
          setFolderError("Ordner konnten nicht geladen werden.");
        }
      }
    };
    void loadFolders();
  }, []);

  const renderStage = () => {
    switch (stage) {
      case 1:
        return (
          <WelcomeScreen
            onNext={() => transitionToStage(2)}
            onInitialize={initRepo}
          />
        );
      case 2:
        return (
          <FolderSelection
            onNext={() => {
              transitionToStage(3);
              void refresh();
            }}
            folderOptions={folderOptions}
            folderError={folderError}
            onImport={importFolders}
            onSeedDemo={async () => {
              const seeded = await seedDemoFiles();
              setFolderOptions(seeded.folders);
              return seeded;
            }}
          />
        );
      case 3:
        return (
          <NodeGraph
            onNext={() => transitionToStage(4)}
            showTutorial
            files={data?.files ?? []}
            views={data?.views ?? []}
          />
        );
      case 4:
        return (
          <SecurityCalibration
            onNext={() => transitionToStage(5)}
            settings={settings}
            onUpdateSettings={updateSecuritySettings}
          />
        );
      case 5:
        return (
          <AhaTutorial
            onComplete={handleComplete}
            files={data?.files ?? []}
            views={data?.views ?? []}
            projects={data?.projects ?? []}
            highlightedFileId={data?.highlightedFileId ?? null}
            onAssignProject={assignToProject}
          />
        );
      case "complete":
        return (
          <div className="flex items-center justify-center min-h-screen">
            <div className="text-center animate-fade-up">
              <h1 className="text-4xl font-bold tracking-tighter mb-4 text-foreground">
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
        <div className="fixed top-6 left-1/2 -translate-x-1/2 z-50 flex gap-2">
          {[1, 2, 3, 4, 5].map((s) => (
            <div
              key={s}
              className={cn(
                "w-2 h-2 rounded-full transition-all duration-500",
                s === stage
                  ? "w-8 bg-primary"
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
