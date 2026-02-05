import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { useViews } from "@/hooks/useViews";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { cn } from "@/lib/utils";
import { Plus, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import SizeableButton from "../ui/sizeable-button";
import { AlertDialog } from "../ui/alert-dialog";
import {
  Popover,
  PopoverArrow,
  PopoverContent,
  PopoverTrigger,
} from "../ui/popover";

interface ObjectDetailPanelProps {
  object: ObjectInfo;
  currentViewId?: string;
  onClose: () => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  onViewSelect: (viewId: string) => void;
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

export const ObjectDetailPanel = ({
  object,
  currentViewId,
  onClose,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  onViewSelect,
}: ObjectDetailPanelProps) => {
  const [trustValue, setTrustValue] = useState<number>(object.trustLevel ?? 70);

  useEffect(() => {
    setTrustValue(object.trustLevel ?? 70);
  }, [object.id, object.trustLevel]);

  const clampedTrust = useMemo(
    () => Math.max(0, Math.min(100, trustValue)),
    [trustValue],
  );

  return (
    <aside className="w-80 border-l border-border/50 bg-background/80 flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border/50">
        <div>
          <p className="text-xs text-foreground/75">Details</p>
          <h2 className="text-sm font-semibold truncate" title={object.name}>
            {object.name}
          </h2>
        </div>
        <Button variant="ghost" size="icon" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <div className="flex-1 overflow-auto px-4 py-4 space-y-6">
        <section className="space-y-2">
          <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
            Informationen
          </h3>
          <div className="space-y-2 text-xs">
            <div className="flex items-center justify-between">
              <span className="text-foreground/75">Typ</span>
              <span className="font-medium uppercase">
                {object.extension || "—"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-foreground/75">Größe</span>
              <span className="font-medium">
                {formatBytes(object.sizeBytes)}
              </span>
            </div>
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
            <ViewsInspector
              value={object.views}
              currentViewId={currentViewId}
              onViewSelect={onViewSelect}
            />
          </div>
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
              Eigenschaften
            </h3>
            <SizeableButton
              icon={<Plus className="w-4 h-4" />}
              label="Hinzufügen"
              onClick={() => onRequestAddTag(object)}
              baseSize="xsmall"
              className="text-xs"
              variant="link"
              expandDirection="left"
            />
          </div>
          {object.tags.length === 0 ? (
            <p className="text-xs text-foreground/75">
              Noch keine Eigenschaften zugewiesen.
            </p>
          ) : (
            <div className="flex flex-wrap gap-2">
              {object.tags.map((tag) => (
                <Badge
                  key={`${tag.key}:${tag.value}`}
                  variant="secondary"
                  className="group"
                >
                  <span className="text-[11px]">
                    {tag.key}:{tag.value}
                  </span>
                  <span className="inline-flex w-0 overflow-hidden transition-[width] duration-200 ease-in-out group-hover:w-4">
                    <button
                      className="ml-1 rounded-full p-0.5 text-foreground/75 hover:text-foreground opacity-0 transition-opacity duration-200 ease-in-out group-hover:opacity-100"
                      onClick={(e) => {
                        e.stopPropagation();
                        onRemoveTag(object, tag);
                      }}
                      aria-label={`Entfernen ${tag.key}:${tag.value}`}
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </span>
                </Badge>
              ))}
            </div>
          )}
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
              Sicherheitsgrad
            </h3>
            <span
              className={cn(
                "text-xs font-semibold",
                clampedTrust >= 85
                  ? "text-green-500"
                  : clampedTrust >= 65
                    ? "text-yellow-500"
                    : clampedTrust >= 40
                      ? "text-orange-500"
                      : "text-red-500",
              )}
            >
              {clampedTrust}%
            </span>
          </div>
          <div className="space-y-2">
            <Slider
              value={[clampedTrust]}
              min={0}
              max={100}
              step={5}
              onValueChange={(value) => setTrustValue(value[0])}
              onValueCommit={(value) => onSetTrust(object, value[0])}
            />
          </div>
        </section>
      </div>
    </aside>
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
