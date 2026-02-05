import { cn } from "@/lib/utils";
import { useViewObjects } from "@/hooks/useViewObjects";
import { HardDrive, CheckCircle } from "lucide-react";

interface StatusBarProps {
  viewName?: string;
  selectedCount: number;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export const StatusBar = ({ viewName, selectedCount }: StatusBarProps) => {
  const { data: objects } = useViewObjects(viewName || "all-objects");

  const objectCount = objects?.length || 0;
  const totalSize = objects?.reduce((acc, obj) => acc + obj.sizeBytes, 0) || 0;

  return (
    <div className="h-6 flex-shrink-0 border-t border-border/50 flex items-center gap-4 px-4 text-xs text-muted-foreground bg-background/50">
      {/* Object count */}
      <span>
        {objectCount} {objectCount === 1 ? "object" : "objects"}
      </span>

      {/* Selection count */}
      {selectedCount > 0 && (
        <span className="text-primary">
          {selectedCount} selected
        </span>
      )}

      {/* Spacer */}
      <div className="flex-1" />

      {/* Total size */}
      <div className="flex items-center gap-1.5">
        <HardDrive className="w-3 h-3" />
        <span>{formatBytes(totalSize)}</span>
      </div>

      {/* Connection status */}
      <div className="flex items-center gap-1.5">
        <CheckCircle className="w-3 h-3 text-green-500" />
        <span>Connected</span>
      </div>
    </div>
  );
};
