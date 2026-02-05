import { useEffect, useMemo, useState } from "react";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { cn } from "@/lib/utils";
import { X } from "lucide-react";

interface ObjectDetailPanelProps {
  object: ObjectInfo;
  onClose: () => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
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

export const ObjectDetailPanel = ({
  object,
  onClose,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
}: ObjectDetailPanelProps) => {
  const [trustValue, setTrustValue] = useState<number>(
    object.trustLevel ?? 70
  );

  useEffect(() => {
    setTrustValue(object.trustLevel ?? 70);
  }, [object.id, object.trustLevel]);

  const clampedTrust = useMemo(
    () => Math.max(0, Math.min(100, trustValue)),
    [trustValue]
  );

  return (
    <aside className="w-80 border-l border-border/50 bg-background/80 flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border/50">
        <div>
          <p className="text-xs text-muted-foreground">Details</p>
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
          <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            File info
          </h3>
          <div className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Type</span>
              <span className="font-medium uppercase">
                {object.extension || "—"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Size</span>
              <span className="font-medium">{formatBytes(object.sizeBytes)}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Created</span>
              <span className="font-medium">
                {formatDate(object.createdAt)}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Modified</span>
              <span className="font-medium">
                {formatDate(object.modifiedAt)}
              </span>
            </div>
          </div>
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
              Tags
            </h3>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-xs"
              onClick={() => onRequestAddTag(object)}
            >
              Add
            </Button>
          </div>
          {object.tags.length === 0 ? (
            <p className="text-xs text-muted-foreground">
              No tags assigned yet.
            </p>
          ) : (
            <div className="flex flex-wrap gap-2">
              {object.tags.map((tag) => (
                <Badge
                  key={`${tag.key}:${tag.value}`}
                  variant="secondary"
                  className="group pr-1"
                >
                  <span className="text-[11px]">
                    {tag.key}:{tag.value}
                  </span>
                  <button
                    className="ml-1 rounded-full p-0.5 text-muted-foreground opacity-0 transition group-hover:opacity-100"
                    onClick={() => onRemoveTag(object, tag)}
                    aria-label={`Remove ${tag.key}:${tag.value}`}
                  >
                    <X className="w-3 h-3" />
                  </button>
                </Badge>
              ))}
            </div>
          )}
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
              Trust score
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
                  : "text-red-500"
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
            <div className="flex items-center gap-2">
              <Input
                type="number"
                min={0}
                max={100}
                value={clampedTrust}
                onChange={(e) => {
                  const parsed = Number(e.target.value);
                  setTrustValue(Number.isFinite(parsed) ? parsed : 0);
                }}
                className="h-8 w-20 text-xs"
              />
              <Button
                variant="secondary"
                size="sm"
                className="h-8 text-xs"
                onClick={() => onSetTrust(object, clampedTrust)}
              >
                Apply
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-8 text-xs"
                onClick={() => onSetTrust(object, null)}
              >
                Clear
              </Button>
            </div>
          </div>
        </section>
      </div>
    </aside>
  );
};
