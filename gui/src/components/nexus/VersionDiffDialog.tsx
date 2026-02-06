import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { DiffResult, ObjectInfo, VersionInfo } from "@/lib/lfs";
import { diffVersions } from "@/lib/lfs";
import { cn } from "@/lib/utils";
import { GitCompareArrows } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

interface VersionDiffDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  object: ObjectInfo;
  versionA: VersionInfo | null;
  versionB: VersionInfo | null;
}

interface DiffLine {
  type: "context" | "addition" | "deletion" | "header";
  content: string;
  lineNumber?: { left?: number; right?: number };
}

function parseDiff(unified: string): DiffLine[] {
  const lines: DiffLine[] = [];
  let leftLine = 0;
  let rightLine = 0;

  for (const raw of unified.split("\n")) {
    if (raw.startsWith("---") || raw.startsWith("+++")) {
      lines.push({ type: "header", content: raw });
    } else if (raw.startsWith("@@")) {
      const match = raw.match(/@@ -(\d+)/);
      if (match) {
        leftLine = parseInt(match[1], 10) - 1;
      }
      const matchRight = raw.match(/\+(\d+)/);
      if (matchRight) {
        rightLine = parseInt(matchRight[1], 10) - 1;
      }
      lines.push({ type: "header", content: raw });
    } else if (raw.startsWith("-")) {
      leftLine++;
      lines.push({
        type: "deletion",
        content: raw.slice(1),
        lineNumber: { left: leftLine },
      });
    } else if (raw.startsWith("+")) {
      rightLine++;
      lines.push({
        type: "addition",
        content: raw.slice(1),
        lineNumber: { right: rightLine },
      });
    } else if (raw.startsWith(" ")) {
      leftLine++;
      rightLine++;
      lines.push({
        type: "context",
        content: raw.slice(1),
        lineNumber: { left: leftLine, right: rightLine },
      });
    }
  }

  return lines;
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

export const VersionDiffDialog = ({
  open,
  onOpenChange,
  object,
  versionA,
  versionB,
}: VersionDiffDialogProps) => {
  const [diff, setDiff] = useState<DiffResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [parsedLines, setParsedLines] = useState<DiffLine[]>([]);

  useEffect(() => {
    if (open && versionA && versionB) {
      setLoading(true);
      setDiff(null);
      setParsedLines([]);
      diffVersions(object.id, versionA.id, versionB.id)
        .then((result) => {
          setDiff(result);
          if (result.unifiedDiff) {
            setParsedLines(parseDiff(result.unifiedDiff));
          }
        })
        .catch((err) => {
          toast.error(
            err instanceof Error ? err.message : "Diff konnte nicht berechnet werden",
          );
        })
        .finally(() => setLoading(false));
    }
  }, [open, object.id, versionA, versionB]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <GitCompareArrows className="w-4 h-4" />
            Vergleich: {object.name}
          </DialogTitle>
          {versionA && versionB && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground pt-1">
              <Badge variant="outline" className="text-[10px]">
                v{versionA.number}
              </Badge>
              <span>vs</span>
              <Badge variant="outline" className="text-[10px]">
                v{versionB.number}
              </Badge>
            </div>
          )}
        </DialogHeader>

        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
              Diff wird berechnet...
            </div>
          ) : !diff ? (
            <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
              Keine Daten vorhanden.
            </div>
          ) : diff.identical ? (
            <div className="flex flex-col items-center justify-center py-12 gap-2">
              <p className="text-sm text-muted-foreground">
                Die Versionen sind identisch.
              </p>
              <div className="flex items-center gap-4 text-xs text-muted-foreground">
                <span>{formatBytes(diff.leftSize)}</span>
                <span>=</span>
                <span>{formatBytes(diff.rightSize)}</span>
              </div>
            </div>
          ) : diff.isBinary ? (
            <div className="flex flex-col items-center justify-center py-12 gap-2">
              <p className="text-sm text-muted-foreground">
                Binärdateien können nicht als Text verglichen werden.
              </p>
              <div className="flex items-center gap-4 text-xs text-muted-foreground">
                <span>v{versionA?.number}: {formatBytes(diff.leftSize)}</span>
                <span>vs</span>
                <span>v{versionB?.number}: {formatBytes(diff.rightSize)}</span>
              </div>
            </div>
          ) : (
            <div className="rounded-md border border-border/50 overflow-hidden font-mono text-xs">
              {parsedLines.map((line, idx) => (
                <div
                  key={idx}
                  className={cn(
                    "flex min-h-[22px]",
                    line.type === "addition" &&
                      "bg-green-500/10 text-green-700 dark:text-green-400",
                    line.type === "deletion" &&
                      "bg-red-500/10 text-red-700 dark:text-red-400",
                    line.type === "header" &&
                      "bg-blue-500/10 text-blue-700 dark:text-blue-400 font-semibold",
                    line.type === "context" && "text-foreground/75",
                  )}
                >
                  <span className="w-10 shrink-0 text-right pr-1 text-[10px] text-muted-foreground/50 select-none border-r border-border/30">
                    {line.lineNumber?.left ?? ""}
                  </span>
                  <span className="w-10 shrink-0 text-right pr-1 text-[10px] text-muted-foreground/50 select-none border-r border-border/30">
                    {line.lineNumber?.right ?? ""}
                  </span>
                  <span className="w-5 shrink-0 text-center select-none">
                    {line.type === "addition"
                      ? "+"
                      : line.type === "deletion"
                        ? "-"
                        : line.type === "header"
                          ? ""
                          : " "}
                  </span>
                  <span className="flex-1 whitespace-pre-wrap break-all pl-1 pr-2">
                    {line.content}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};
