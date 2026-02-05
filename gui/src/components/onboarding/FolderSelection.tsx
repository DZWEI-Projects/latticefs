import { useState, useEffect, useMemo } from "react";
import { GlassCard } from "./ui/GlassCard";
import { AnimatedButton } from "./ui/AnimatedButton";
import { ToggleSwitch } from "./ui/ToggleSwitch";
import { ParticleBackground } from "./ui/ParticleBackground";
import { cn } from "@/lib/utils";
import { FileText, Download, Image, Code, FolderPlus, Loader2 } from "lucide-react";
import type { FolderOption } from "@/lib/latticeApi";

interface FolderSelectionProps {
  onNext: () => void;
  folderOptions: FolderOption[];
  folderError: string | null;
  onImport: (folderIds: string[]) => Promise<ImportResponse>;
  onSeedDemo: () => Promise<{ demoRoot: string; folders: FolderOption[] }>;
}

const iconMap: Record<string, typeof FileText> = {
  documents: FileText,
  downloads: Download,
  bilder: Image,
  projekte: Code,
  demo: FolderPlus,
};

export const FolderSelection = ({
  onNext,
  folderOptions,
  folderError,
  onImport,
  onSeedDemo,
}: FolderSelectionProps) => {
  const [selectedFolders, setSelectedFolders] = useState<Set<string>>(
    new Set()
  );
  const [isScanning, setIsScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState(0);
  const [currentScanFolder, setCurrentScanFolder] = useState("");
  const [seedMessage, setSeedMessage] = useState<string | null>(null);
  const [seedError, setSeedError] = useState<string | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [isSeeding, setIsSeeding] = useState(false);

  const orderedFolders = useMemo(
    () => folderOptions.filter((folder) => folder.exists),
    [folderOptions]
  );

  useEffect(() => {
    if (folderOptions.length === 0 || selectedFolders.size > 0) return;
    setSelectedFolders(
      new Set(folderOptions.filter((f) => f.defaultSelected && f.exists).map((f) => f.id))
    );
  }, [folderOptions, selectedFolders.size]);

  const toggleFolder = (id: string) => {
    setSelectedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleScan = async () => {
    if (selectedFolders.size === 0) return;
    setImportError(null);
    setIsScanning(true);
    setScanProgress(0);
    const folderIds = Array.from(selectedFolders);

    try {
      await onImport(folderIds);
      setScanProgress(100);
      setTimeout(onNext, 500);
    } catch (err) {
      setIsScanning(false);
      if (err instanceof Error) {
        setImportError(err.message);
      } else {
        setImportError("Import fehlgeschlagen.");
      }
    }
  };

  // Simulate scanning animation
  useEffect(() => {
    if (!isScanning) return;

    const folders = Array.from(selectedFolders);
    let currentIndex = 0;
    let progress = 0;

    const interval = setInterval(() => {
      progress += Math.random() * 15 + 5;
      
      if (progress >= 100) {
        progress = 100;
        clearInterval(interval);
      }
      
      setScanProgress(Math.min(progress, 100));
      
      // Update current folder being scanned
      const folderIndex = Math.floor((progress / 100) * folders.length);
      if (folderIndex !== currentIndex && folderIndex < folders.length) {
        currentIndex = folderIndex;
        const folder = folderOptions.find((f) => f.id === folders[folderIndex]);
        if (folder) setCurrentScanFolder(folder.name);
      }
    }, 256);

    // Set initial folder
    const firstFolder = folderOptions.find((f) => f.id === folders[0]);
    if (firstFolder) setCurrentScanFolder(firstFolder.name);

    return () => clearInterval(interval);
  }, [isScanning, selectedFolders, folderOptions]);

  const handleSeedDemo = async () => {
    setSeedMessage(null);
    setSeedError(null);
    setIsSeeding(true);
    try {
      const seeded = await onSeedDemo();
      setSeedMessage(`Demo-Dateien erstellt: ${seeded.demoRoot}`);
      const demoFolder = seeded.folders.find((folder) => folder.isDemo);
      if (demoFolder) {
        setSelectedFolders((prev) => new Set([...prev, demoFolder.id]));
      }
    } catch (err) {
      if (err instanceof Error) {
        setSeedError(err.message);
      } else {
        setSeedError("Demo-Dateien konnten nicht erstellt werden.");
      }
    } finally {
      setIsSeeding(false);
    }
  };

  if (isScanning) {
    return (
      <div className="relative flex flex-col items-center justify-center min-h-screen px-6">
        <ParticleBackground particleCount={40} />
        
        <div className="relative z-10 flex flex-col items-center max-w-md text-center">
          {/* Scanning animation */}
          <div className="relative mb-8">
            <div 
              className="w-32 h-32 rounded-full border-4 border-primary/20 flex items-center justify-center"
              style={{
                background: `conic-gradient(hsl(var(--primary)) ${scanProgress * 3.6}deg, transparent 0deg)`,
              }}
            >
              <div className="w-28 h-28 rounded-full bg-background flex items-center justify-center">
                <Loader2 className="w-12 h-12 text-primary animate-spin" />
              </div>
            </div>
          </div>
          
          <h2 className="text-2xl font-bold tracking-tight mb-2">
            Analysiere deine Dateien
          </h2>
          <p className="text-muted-foreground mb-4">
            Scanne: {currentScanFolder}
          </p>
          <div className="w-full h-2 rounded-full bg-muted overflow-hidden">
            <div 
              className="h-full bg-primary transition-all duration-300 ease-out"
              style={{ width: `${scanProgress}%` }}
            />
          </div>
          <p className="text-sm text-muted-foreground mt-2">
            {Math.round(scanProgress)}%
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="relative flex flex-col items-center justify-center min-h-screen px-6">
      <ParticleBackground particleCount={40} />
      
      <div className="relative z-10 flex flex-col items-center max-w-lg w-full">
        {/* Header */}
        <div 
          className="text-center mb-10 opacity-0 animate-fade-up"
          style={{ animationFillMode: "forwards" }}
        >
          <h2 className="text-3xl md:text-4xl font-bold tracking-tighter mb-3">
            Was soll NeuralFS über dich lernen?
          </h2>
          <p className="text-muted-foreground">
            Wähle die Ordner aus, die du einbinden möchtest.
          </p>
        </div>
        
        {/* Folder grid */}
        <div className="grid grid-cols-2 gap-4 w-full mb-6">
          {orderedFolders.map((folder, index) => {
            const Icon = iconMap[folder.id] ?? FolderPlus;
            return (
            <GlassCard
              key={folder.id}
              delay={200 + index * 100}
              hover
              className={cn(
                "cursor-pointer transition-all duration-300",
                selectedFolders.has(folder.id) && "border-primary/40 glow-primary"
              )}
            >
              <div 
                className="flex items-center justify-between"
                onClick={() => toggleFolder(folder.id)}
              >
                <div className="flex items-center gap-3">
                  <div className={cn(
                    "p-2 rounded-lg transition-colors duration-300",
                    selectedFolders.has(folder.id) ? "bg-primary/20" : "bg-muted"
                  )}>
                    <Icon className={cn(
                      "w-5 h-5 transition-colors duration-300",
                      selectedFolders.has(folder.id) ? "text-primary" : "text-muted-foreground"
                    )} />
                  </div>
                  <div>
                    <span className="font-medium">{folder.name}</span>
                    <p className="text-xs text-muted-foreground/70">{folder.path}</p>
                  </div>
                </div>
                <ToggleSwitch
                  checked={selectedFolders.has(folder.id)}
                  onChange={() => toggleFolder(folder.id)}
                />
              </div>
            </GlassCard>
            );
          })}
          
          {/* Add more folders option */}
          <GlassCard
            delay={600}
            hover
            className="cursor-pointer col-span-2 opacity-70 hover:opacity-100"
          >
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-muted">
                <FolderPlus className="w-5 h-5 text-muted-foreground" />
              </div>
              <span className="text-muted-foreground">Weitere Ordner auswählen...</span>
            </div>
          </GlassCard>
        </div>
        
        {/* Reassurance text */}
        <p 
          className="text-sm text-muted-foreground/70 text-center mb-8 opacity-0 animate-fade-up"
          style={{ animationDelay: "700ms", animationFillMode: "forwards" }}
        >
          Keine Sorge — nichts wird geändert oder verschoben.
        </p>
        
        {/* CTA Button */}
        <div 
          className="opacity-0 animate-fade-up"
          style={{ animationDelay: "800ms", animationFillMode: "forwards" }}
        >
          <AnimatedButton 
            onClick={handleScan}
            disabled={selectedFolders.size === 0 || isScanning}
          >
            Ausgewählte Ordner scannen
          </AnimatedButton>
          {importError && (
            <p className="mt-3 text-sm text-warning text-center">
              {importError}
            </p>
          )}
        </div>

        <div className="mt-4 flex flex-col items-center gap-2">
          <AnimatedButton
            onClick={handleSeedDemo}
            disabled={isSeeding}
            size="sm"
            className="bg-secondary/30 hover:bg-secondary/50"
          >
            {isSeeding ? "Erstelle Demo-Dateien..." : "Demo-Dateien erstellen"}
          </AnimatedButton>
          {seedMessage && (
            <p className="text-xs text-muted-foreground">{seedMessage}</p>
          )}
          {seedError && (
            <p className="text-xs text-warning">{seedError}</p>
          )}
          {folderError && (
            <p className="text-xs text-warning">{folderError}</p>
          )}
        </div>
      </div>
    </div>
  );
};
