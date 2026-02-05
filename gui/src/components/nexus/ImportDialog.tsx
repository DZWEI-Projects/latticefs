import { useState, useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { pickFiles, pickFolders, importPaths } from "@/lib/lfs";
import type { ImportProgress } from "@/lib/lfs";
import { File, Folder, Loader2, CheckCircle2, XCircle } from "lucide-react";
import { listen } from "@tauri-apps/api/event";

interface ImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportComplete?: () => void;
}

type ImportState = "idle" | "selecting" | "importing" | "success" | "error";

export const ImportDialog = ({
  open,
  onOpenChange,
  onImportComplete,
}: ImportDialogProps) => {
  const queryClient = useQueryClient();
  const [state, setState] = useState<ImportState>("idle");
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [importedCount, setImportedCount] = useState(0);

  // Listen for import progress events
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      try {
        unlisten = await listen<ImportProgress>("import_progress", (event) => {
          setProgress(event.payload);
        });
      } catch {
        // Not in Tauri environment, ignore
      }
    };

    if (state === "importing") {
      setupListener();
    }

    return () => {
      unlisten?.();
    };
  }, [state]);

  const handleSelectFiles = async () => {
    setState("selecting");
    try {
      const paths = await pickFiles();
      if (paths && paths.length > 0) {
        setSelectedPaths((prev) => [...prev, ...paths]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
    setState("idle");
  };

  const handleSelectFolders = async () => {
    setState("selecting");
    try {
      const paths = await pickFolders();
      if (paths && paths.length > 0) {
        setSelectedPaths((prev) => [...prev, ...paths]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
    setState("idle");
  };

  const handleRemovePath = (index: number) => {
    setSelectedPaths((prev) => prev.filter((_, i) => i !== index));
  };

  const handleImport = async () => {
    if (selectedPaths.length === 0) return;

    setState("importing");
    setError(null);
    setProgress(null);

    try {
      const targets = selectedPaths.map((path) => ({
        path,
        tags: [],
      }));

      const result = await importPaths(targets);
      setImportedCount(result.imported);
      setState("success");

      // Invalidate queries to refresh the view
      await queryClient.invalidateQueries({ queryKey: ["views"] });
      await queryClient.invalidateQueries({ queryKey: ["viewObjects"] });

      onImportComplete?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setState("error");
    }
  };

  const handleClose = () => {
    if (state === "importing") return;

    setSelectedPaths([]);
    setProgress(null);
    setError(null);
    setImportedCount(0);
    setState("idle");
    onOpenChange(false);
  };

  const renderContent = () => {
    if (state === "importing") {
      return (
        <div className="py-8 space-y-4">
          <div className="flex items-center justify-center">
            <Loader2 className="w-8 h-8 animate-spin text-primary" />
          </div>
          <p className="text-center text-sm text-muted-foreground">
            Importing files...
          </p>
          {progress && (
            <div className="space-y-2">
              <Progress value={(progress.current / progress.total) * 100} />
              <p className="text-center text-xs text-muted-foreground">
                {progress.current} of {progress.total} files
              </p>
            </div>
          )}
        </div>
      );
    }

    if (state === "success") {
      return (
        <div className="py-8 space-y-4">
          <div className="flex items-center justify-center">
            <CheckCircle2 className="w-12 h-12 text-green-500" />
          </div>
          <p className="text-center text-sm">
            Successfully imported {importedCount} object
            {importedCount !== 1 ? "s" : ""}
          </p>
        </div>
      );
    }

    if (state === "error") {
      return (
        <div className="py-8 space-y-4">
          <div className="flex items-center justify-center">
            <XCircle className="w-12 h-12 text-destructive" />
          </div>
          <p className="text-center text-sm text-destructive">{error}</p>
        </div>
      );
    }

    return (
      <div className="space-y-4 py-4">
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={handleSelectFiles}
            disabled={state === "selecting"}
            className="flex-1"
          >
            <File className="w-4 h-4 mr-2" />
            Add Files
          </Button>
          <Button
            variant="outline"
            onClick={handleSelectFolders}
            disabled={state === "selecting"}
            className="flex-1"
          >
            <Folder className="w-4 h-4 mr-2" />
            Add Folders
          </Button>
        </div>

        {selectedPaths.length > 0 && (
          <div className="border rounded-lg max-h-48 overflow-y-auto">
            {selectedPaths.map((path, index) => (
              <div
                key={index}
                className="flex items-center justify-between px-3 py-2 border-b last:border-b-0 text-sm"
              >
                <span className="truncate flex-1 font-mono text-xs">
                  {path}
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleRemovePath(index)}
                  className="h-6 w-6 p-0 ml-2"
                >
                  ×
                </Button>
              </div>
            ))}
          </div>
        )}

        {selectedPaths.length === 0 && (
          <div className="border border-dashed rounded-lg py-8 text-center text-muted-foreground">
            <p className="text-sm">No files or folders selected</p>
            <p className="text-xs mt-1">
              Click the buttons above to add items to import
            </p>
          </div>
        )}
      </div>
    );
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Import Files</DialogTitle>
          <DialogDescription>
            Add files and folders to import into LatticeFS. They will be
            content-addressed and available across all your views.
          </DialogDescription>
        </DialogHeader>

        {renderContent()}

        <DialogFooter>
          {state === "idle" && (
            <>
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                onClick={handleImport}
                disabled={selectedPaths.length === 0}
              >
                Import {selectedPaths.length > 0 && `(${selectedPaths.length})`}
              </Button>
            </>
          )}
          {(state === "success" || state === "error") && (
            <Button onClick={handleClose}>Done</Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
