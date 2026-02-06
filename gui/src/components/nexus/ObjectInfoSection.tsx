import { Badge } from "@/components/ui/badge";
import type { ObjectInfo } from "@/lib/lfs";
import { VERSION_STATE_LABELS, type VersionState } from "@/lib/lfs";
import { cn } from "@/lib/utils";
import { formatMimeType } from "@/lib/metadataDisplay";
import { useState } from "react";
import { useViews } from "@/hooks/useViews";
import {
  Popover,
  PopoverArrow,
  PopoverContent,
  PopoverTrigger,
} from "../ui/popover";

interface ObjectInfoSectionProps {
  object: ObjectInfo;
  currentViewId?: string;
  mimeType?: string | null;
  onCopyId: () => void;
  onViewSelect: (viewId: string) => void;
  onOpenVersions?: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`.replace(
    ".",
    ",",
  );
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatViews(views: string[]): string {
  return views.length === 1
    ? "einer Perspektive"
    : `${views.length} Perspektiven`;
}

function formatObjectType(type: ObjectInfo["objectType"]): string {
  switch (type) {
    case "blob":
      return "Datei";
    case "tree":
      return "Ordner";
    case "commit":
      return "Commit";
    default:
      return type;
  }
}

function stateColor(state: string): string {
  switch (state) {
    case "draft": return "text-blue-500";
    case "review": return "text-yellow-500";
    case "approved": return "text-green-500";
    case "discarded": return "text-muted-foreground";
    case "sealed": return "text-red-500";
    case "archived": return "text-muted-foreground";
    default: return "";
  }
}

export const ObjectInfoSection = ({
  object,
  currentViewId,
  mimeType,
  onCopyId,
  onViewSelect,
  onOpenVersions,
}: ObjectInfoSectionProps) => {
  const stateLabel = VERSION_STATE_LABELS[object.currentVersionState as VersionState] ?? object.currentVersionState;

  return (
    <section className="space-y-2">
      <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
        Informationen
      </h3>
      <div className="space-y-2 text-xs">
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Objekttyp</span>
          <span className="font-medium">{formatMimeType(mimeType)}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Objektart</span>
          <span className="font-medium">{formatObjectType(object.objectType)}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Größe</span>
          <span className="font-medium">
            {formatBytes(object.sizeBytes)}
          </span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Status</span>
          <span className={cn("font-medium", stateColor(object.currentVersionState))}>
            {stateLabel}
            {object.isSealed && " (gesperrt)"}
          </span>
        </div>
        {object.versionCount > 1 && (
          <div className="flex items-center justify-between">
            <span className="text-foreground/75">Versionen</span>
            <button
              type="button"
              className="font-medium hover:text-primary hover:underline transition-colors"
              onClick={onOpenVersions}
            >
              {object.versionCount} Versionen
            </button>
          </div>
        )}
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Erstellt</span>
          <span className="font-medium">
            {formatDate(object.createdAt)}
          </span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Geändert</span>
          <span className="font-medium">
            {formatDate(object.modifiedAt)}
          </span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-foreground/75">Objekt-ID</span>
          <button
            type="button"
            className={cn(
              "font-medium font-mono text-[11px] truncate max-w-[180px]",
              "text-foreground/80 hover:text-primary hover:underline",
              "transition-colors",
            )}
            title="Objekt-ID kopieren"
            onClick={onCopyId}
          >
            {object.id}
          </button>
        </div>
        <ViewsInspector
          value={object.views}
          currentViewId={currentViewId}
          onViewSelect={onViewSelect}
        />
      </div>
    </section>
  );
};

function ViewsInspector({
  value,
  currentViewId,
  onViewSelect,
}: {
  value: string[];
  currentViewId?: string;
  onViewSelect: (viewId: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const { data: views, isLoading } = useViews();

  return (
    <div className="flex items-center justify-between">
      <span className="text-foreground/75">Perspektive</span>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <span
            className="font-medium cursor-pointer hover:text-primary hover:underline"
            onClick={() => setOpen(!open)}
          >
            In {formatViews(value)}
          </span>
        </PopoverTrigger>
        <PopoverContent className="w-72">
          <PopoverArrow />
          <div className="space-y-3">
            <div className="space-y-1">
              <h4 className="text-sm font-semibold">Perspektiven</h4>
              <p className="text-xs text-muted-foreground">
                Dieses Objekt ist in {formatViews(value)} verfügbar.
              </p>
            </div>
            {isLoading ? (
              <p className="text-xs text-muted-foreground">Lädt...</p>
            ) : views && views.length > 0 ? (
              <div className="space-y-1">
                {views
                  .filter((view) => value.includes(view.id))
                  .map((view) => {
                    const isActive = currentViewId === view.id;
                    return (
                      <button
                        key={view.id}
                        type="button"
                        onClick={() => {
                          onViewSelect(view.id);
                          setOpen(false);
                        }}
                        className={cn(
                          "w-full flex items-center justify-between gap-2 rounded-md px-2 py-1 text-xs transition-colors",
                          "hover:bg-muted/60",
                          isActive && "bg-primary/10 text-primary hover:bg-primary/15",
                        )}
                      >
                        <span className="flex-1 text-left truncate">{view.name}</span>
                        <Badge variant="secondary" className="text-[10px]">
                          Enthält
                        </Badge>
                      </button>
                    );
                  })}
              </div>
            ) : (
              <p className="text-xs text-muted-foreground">
                Keine Perspektiven gefunden.
              </p>
            )}
          </div>
        </PopoverContent>
      </Popover>
    </div>
  );
}
