import { useEffect, useMemo, useState } from "react";
import { GlassCard } from "./ui/GlassCard";
import { AnimatedButton } from "./ui/AnimatedButton";
import { ToggleSwitch } from "./ui/ToggleSwitch";
import { ParticleBackground } from "./ui/ParticleBackground";
import { cn } from "@/lib/utils";
import {
  FileText,
  Download,
  Image,
  Code,
  FolderPlus,
  Loader2,
  Plus,
} from "lucide-react";
import {
  createSampleFiles,
  importPaths,
  isTauriApp,
  onImportProgress,
  type ImportSummary,
} from "@/lib/lfs";
import { documentDir, downloadDir, homeDir, join, pictureDir } from "@tauri-apps/api/path";

interface FolderSelectionProps {
  onNext: () => void;
}

interface FolderOption {
  id: string;
  name: string;
  icon: typeof FileText;
  defaultSelected: boolean;
  resolvePath: () => Promise<string | null>;
  tags: string[];
}

const folderOptions: FolderOption[] = [
  {
    id: "dokumente",
    name: "Dokumente",
    icon: FileText,
    defaultSelected: true,
    resolvePath: async () => documentDir(),
    tags: ["source:documents"],
  },
  {
    id: "downloads",
    name: "Downloads",
    icon: Download,
    defaultSelected: true,
    resolvePath: async () => downloadDir(),
    tags: ["source:downloads"],
  },
  {
    id: "bilder",
    name: "Bilder",
    icon: Image,
    defaultSelected: false,
    resolvePath: async () => pictureDir(),
    tags: ["source:pictures"],
  },
  {
    id: "projekte",
    name: "Projekte",
    icon: Code,
    defaultSelected: false,
    resolvePath: async () => {
      const home = await homeDir();
      return home ? join(home, "Projects") : null;
    },
    tags: ["source:projects"],
  },
];

export const FolderSelection = ({ onNext }: FolderSelectionProps) => {
  const [selectedFolders, setSelectedFolders] = useState<Set<string>>(new Set());
  const [folderPaths, setFolderPaths] = useState<Record<string, string>>({});
  const [sampleRoot, setSampleRoot] = useState<string | null>(null);
  const [sampleFiles, setSampleFiles] = useState<string[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState(0);
  const [currentScanFolder, setCurrentScanFolder] = useState("");
  const [importSummary, setImportSummary] = useState<ImportSummary | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [isCreatingSamples, setIsCreatingSamples] = useState(false);
  const [isResolvingPaths, setIsResolvingPaths] = useState(true);

  const availableFolders = useMemo(() => {
    const base = folderOptions.filter((option) => folderPaths[option.id]);
    if (sampleRoot) {
      return [
        ...base,
        {
          id: "samples",
          name: "LatticeFS Samples",
          icon: FolderPlus,
          defaultSelected: true,
          resolvePath: async () => sampleRoot,
          tags: ["source:samples"],
        } satisfies FolderOption,
      ];
    }
    return base;
  }, [folderPaths, sampleRoot]);

  useEffect(() => {
    const resolvePaths = async () => {
      const entries = await Promise.all(
        folderOptions.map(async (option) => {
          try {
            const path = await option.resolvePath();
            return [option.id, path] as const;
          } catch {
            return [option.id, null] as const;
          }
        })
      );
      const resolved: Record<string, string> = {};
      entries.forEach(([id, path]) => {
        if (path) {
          resolved[id] = path;
        }
      });
      setFolderPaths(resolved);
      setSelectedFolders(
        new Set(
          folderOptions
            .filter((option) => option.defaultSelected && resolved[option.id])
            .map((option) => option.id)
        )
      );
      setIsResolvingPaths(false);
    };
    resolvePaths().catch(() => setIsResolvingPaths(false));
  }, []);

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
    if (!isTauriApp()) {
      setImportError("Die Desktop-App ist erforderlich, um Dateien zu importieren.");
      return;
    }
    if (selectedFolders.size === 0) return;
    setIsScanning(true);
    setImportError(null);
    setImportSummary(null);
    setScanProgress(0);

    const selected = availableFolders.filter((folder) =>
      selectedFolders.has(folder.id)
    );
    const targets = selected
      .map((folder) => {
        const path = folder.id === "samples" ? sampleRoot : folderPaths[folder.id];
        if (!path) return null;
        return { path, tags: folder.tags };
      })
      .filter((target): target is { path: string; tags: string[] } => Boolean(target));

    if (targets.length === 0) {
      setIsScanning(false);
      setImportError("Keine gültigen Ordner gefunden.");
      return;
    }

    const unlisten = await onImportProgress((progress) => {
      const nextProgress = progress.total === 0 ? 0 : (progress.current / progress.total) * 100;
      setScanProgress(Math.min(nextProgress, 100));
      setCurrentScanFolder(progress.path);
    });

    try {
      const summary = await importPaths(targets);
      setImportSummary(summary);
      if (summary.failed === 0) {
        setTimeout(onNext, 800);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "Import fehlgeschlagen.";
      setImportError(message);
    } finally {
      await unlisten();
      setIsScanning(false);
    }
  };

  const handleCreateSamples = async () => {
    if (!isTauriApp()) {
      setImportError("Die Desktop-App ist erforderlich, um Beispieldateien zu erstellen.");
      return;
    }
    setIsCreatingSamples(true);
    setImportError(null);
    try {
      const result = await createSampleFiles();
      setSampleRoot(result.root);
      setSampleFiles(result.files);
      setSelectedFolders((prev) => new Set(prev).add("samples"));
    } catch (err) {
      const message = err instanceof Error ? err.message : "Beispieldateien konnten nicht erstellt werden.";
      setImportError(message);
    } finally {
      setIsCreatingSamples(false);
    }
  };

  if (isScanning || importSummary) {
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
          {importSummary && (
            <div className="mt-6 text-sm text-muted-foreground space-y-2">
              <p>
                Importiert: <span className="text-foreground">{importSummary.imported}</span>
              </p>
              <p>
                Fehler: <span className="text-foreground">{importSummary.failed}</span>
              </p>
              {importSummary.failed > 0 && (
                <div className="text-left max-h-32 overflow-y-auto rounded-md border border-muted/40 bg-muted/20 p-3">
                  <ul className="list-disc list-inside space-y-1">
                    {importSummary.errors.map((error) => (
                      <li key={error}>{error}</li>
                    ))}
                  </ul>
                </div>
              )}
              {importSummary.failed > 0 && (
                <AnimatedButton onClick={onNext} size="md">
                  Weiter
                </AnimatedButton>
              )}
            </div>
          )}
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
          {availableFolders.map((folder, index) => (
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
                  <div
                    className={cn(
                      "p-2 rounded-lg transition-colors duration-300",
                      selectedFolders.has(folder.id) ? "bg-primary/20" : "bg-muted"
                    )}
                  >
                    <folder.icon
                      className={cn(
                        "w-5 h-5 transition-colors duration-300",
                        selectedFolders.has(folder.id) ? "text-primary" : "text-muted-foreground"
                      )}
                    />
                  </div>
                  <span className="font-medium">{folder.name}</span>
                </div>
                <ToggleSwitch
                  checked={selectedFolders.has(folder.id)}
                  onChange={() => toggleFolder(folder.id)}
                />
              </div>
            </GlassCard>
          ))}

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

        <div className="flex flex-col items-center gap-3 mb-6">
          <AnimatedButton
            onClick={handleCreateSamples}
            variant="secondary"
            size="md"
            showArrow={false}
            disabled={isCreatingSamples || !isTauriApp()}
          >
            <Plus className="w-4 h-4" />
            {isCreatingSamples ? "Erstelle Dateien..." : "Beispieldateien erstellen"}
          </AnimatedButton>
          {sampleRoot && (
            <p className="text-xs text-muted-foreground text-center">
              Beispielordner: {sampleRoot}
            </p>
          )}
          {sampleFiles.length > 0 && (
            <p className="text-xs text-muted-foreground text-center">
              {sampleFiles.length} Dateien bereit für den Import.
            </p>
          )}
          {importError && (
            <p className="text-xs text-warning text-center">{importError}</p>
          )}
          {isResolvingPaths && (
            <p className="text-xs text-muted-foreground text-center">
              Ordnerpfade werden geladen...
            </p>
          )}
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
            disabled={selectedFolders.size === 0 || isResolvingPaths || !isTauriApp()}
          >
            Ausgewählte Ordner scannen
          </AnimatedButton>
        </div>
      </div>
    </div>
  );
};
