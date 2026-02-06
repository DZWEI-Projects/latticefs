import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useViewObjects } from "@/hooks/useViewObjects";
import { useMountStatus } from "@/hooks/useMountStatus";
import { mountRepo, onMountError, onMountStopped, unmountRepo } from "@/lib/lfs";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { HardDrive, CheckCircle, Plug, Unplug } from "lucide-react";
import { toast } from "sonner";

interface StatusBarProps {
  viewId?: string;
  selectedCount: number;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export const StatusBar = ({ viewId, selectedCount }: StatusBarProps) => {
  const { data: objects } = useViewObjects(viewId || "all-objects");
  const { data: mountStatus, isLoading: mountLoading } = useMountStatus();
  const queryClient = useQueryClient();
  const [mountBusy, setMountBusy] = useState(false);

  const objectCount = objects?.length || 0;
  const totalSize = objects?.reduce((acc, obj) => acc + obj.sizeBytes, 0) || 0;
  const mountAvailable = mountStatus?.available ?? false;
  const mountMounted = mountStatus?.mounted ?? false;
  const mountPoint = mountStatus?.mountPoint ?? "—";
  const mountReason = mountStatus?.reason ?? "FUSE ist nicht verfügbar.";
  const mountUnavailable = !mountAvailable && !mountLoading;
  const mountLabel = mountLoading
    ? "Mount wird geprüft..."
    : mountMounted
      ? "Eingehängt"
      : mountAvailable
        ? "Nicht eingehängt"
        : "FUSE nicht verfügbar";
  const actionLabel = mountMounted ? "Aushängen" : "Einhängen";

  const refreshMountStatus = () => {
    queryClient.invalidateQueries({ queryKey: ["mount-status"] });
  };

  useEffect(() => {
    let active = true;
    let unlistenError: (() => void) | null = null;
    let unlistenStopped: (() => void) | null = null;

    onMountError((message) => {
      toast.error(message);
      refreshMountStatus();
    }).then((unlisten) => {
      if (!active) {
        unlisten();
      } else {
        unlistenError = unlisten;
      }
    });

    onMountStopped(() => {
      refreshMountStatus();
    }).then((unlisten) => {
      if (!active) {
        unlisten();
      } else {
        unlistenStopped = unlisten;
      }
    });

    return () => {
      active = false;
      unlistenError?.();
      unlistenStopped?.();
    };
  }, [queryClient]);

  const handleMountAction = async () => {
    if (!mountAvailable || mountBusy || mountLoading) return;
    setMountBusy(true);
    try {
      if (mountMounted) {
        await unmountRepo();
        toast.success("LatticeFS ausgehängt");
      } else {
        await mountRepo();
        toast.success("LatticeFS-Mount gestartet");
      }
      refreshMountStatus();
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Mount-Aktion fehlgeschlagen",
      );
    } finally {
      setMountBusy(false);
    }
  };

  return (
    <div className="h-6 flex-shrink-0 border-t border-border/50 flex items-center gap-4 px-4 text-xs text-muted-foreground bg-background/50">
      {/* Object count */}
      <span>
        {objectCount} {objectCount === 1 ? "Objekt" : "Objekte"}
      </span>

      {/* Selection count */}
      {selectedCount > 0 && (
        <span className="text-primary">
          {selectedCount} ausgewählt
        </span>
      )}

      {/* Spacer */}
      <div className="flex-1" />

      {/* Total size */}
      <div className="flex items-center gap-1.5">
        <HardDrive className="w-3 h-3" />
        <span>{formatBytes(totalSize)}</span>
      </div>

      {/* Mount status */}
      <div className="flex items-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center gap-1.5">
              {mountMounted ? (
                <Plug className="w-3 h-3 text-emerald-500" />
              ) : (
                <Unplug className="w-3 h-3 text-muted-foreground" />
              )}
              <span>{mountLabel}</span>
            </div>
          </TooltipTrigger>
          <TooltipContent side="top" align="end">
            <div className="space-y-1 text-xs">
              <div className="text-muted-foreground">Mount-Punkt</div>
              <div className="font-medium text-foreground">{mountPoint}</div>
              {mountUnavailable && (
                <div className="text-destructive">{mountReason}</div>
              )}
            </div>
          </TooltipContent>
        </Tooltip>
        {mountAvailable || mountLoading ? (
          <Button
            variant="ghost"
            size="xs"
            onClick={handleMountAction}
            disabled={mountBusy || mountLoading}
          >
            {actionLabel}
          </Button>
        ) : (
          <Tooltip>
            <TooltipTrigger asChild>
              <span>
                <Button variant="ghost" size="xs" disabled>
                  {actionLabel}
                </Button>
              </span>
            </TooltipTrigger>
            <TooltipContent side="top">FUSE ist nicht verfügbar.</TooltipContent>
          </Tooltip>
        )}
      </div>

      {/* Connection status */}
      <div className="flex items-center gap-1.5">
        <CheckCircle className="w-3 h-3 text-green-500" />
        <span>Verbunden</span>
      </div>
    </div>
  );
};
