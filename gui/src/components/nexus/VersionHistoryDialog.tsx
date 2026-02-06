import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { ObjectInfo, VersionInfo, VersionState } from "@/lib/lfs";
import {
  VERSION_STATE_LABELS,
  VERSION_STATE_TRANSITIONS,
  getObjectVersions,
  setVersionMessage,
  setVersionState,
  exportObjectVersion,
  checkoutObjectVersion,
} from "@/lib/lfs";
import { cn } from "@/lib/utils";
import {
  ArrowDownToLine,
  Check,
  Clock,
  GitCompareArrows,
  Lock,
  MessageSquareText,
  Pencil,
  RotateCcw,
  X,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

interface VersionHistoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  object: ObjectInfo;
  onOpenDiff: (versionA: VersionInfo, versionB: VersionInfo) => void;
  onOpenEditor: (object: ObjectInfo) => void;
  onObjectUpdated: (updated?: ObjectInfo) => void;
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

function stateVariant(state: string): "default" | "secondary" | "destructive" | "outline" {
  switch (state) {
    case "approved": return "default";
    case "sealed": return "destructive";
    case "review": return "secondary";
    default: return "outline";
  }
}

export const VersionHistoryDialog = ({
  open,
  onOpenChange,
  object,
  onOpenDiff,
  onOpenEditor,
  onObjectUpdated,
}: VersionHistoryDialogProps) => {
  const [versions, setVersions] = useState<VersionInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [diffSelectMode, setDiffSelectMode] = useState(false);
  const [diffSelection, setDiffSelection] = useState<VersionInfo[]>([]);
  const [editingVersionId, setEditingVersionId] = useState<string | null>(null);
  const [messageDraft, setMessageDraft] = useState("");
  const [messageSaving, setMessageSaving] = useState(false);

  const loadVersions = useCallback(async () => {
    setLoading(true);
    try {
      const result = await getObjectVersions(object.id);
      setVersions(result);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Versionen konnten nicht geladen werden",
      );
    } finally {
      setLoading(false);
    }
  }, [object.id]);

  useEffect(() => {
    if (open) {
      loadVersions();
      setDiffSelectMode(false);
      setDiffSelection([]);
      setEditingVersionId(null);
      setMessageDraft("");
    }
  }, [open, loadVersions]);

  const handleStateChange = async (version: VersionInfo, newState: VersionState) => {
    try {
      const updated = await setVersionState(object.id, version.id, newState);
      setVersions((prev) =>
        prev.map((v) => (v.id === updated.id ? updated : v)),
      );
      onObjectUpdated();
      toast.success(
        `Status geändert: ${VERSION_STATE_LABELS[newState]}`,
      );
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Status konnte nicht geändert werden",
      );
    }
  };

  const handleExport = async (version: VersionInfo) => {
    try {
      await exportObjectVersion(object.id, version.id);
      toast.success("Version exportiert");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Export fehlgeschlagen",
      );
    }
  };

  const handleCheckout = async (version: VersionInfo) => {
    try {
      const updated = await checkoutObjectVersion(object.id, version.id);
      setVersions((prev) =>
        prev.map((v) => ({ ...v, isCurrent: v.id === version.id })),
      );
      onObjectUpdated(updated);
      toast.success(`Version v${version.number} aktiviert`);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Aktivierung fehlgeschlagen",
      );
    }
  };

  const handleDiffSelect = (version: VersionInfo) => {
    setDiffSelection((prev) => {
      if (prev.find((v) => v.id === version.id)) {
        return prev.filter((v) => v.id !== version.id);
      }
      if (prev.length >= 2) {
        return [prev[1], version];
      }
      return [...prev, version];
    });
  };

  const startMessageEdit = (version: VersionInfo) => {
    setEditingVersionId(version.id);
    setMessageDraft(version.commitMessage ?? "");
  };

  const cancelMessageEdit = () => {
    setEditingVersionId(null);
    setMessageDraft("");
  };

  const handleMessageSave = async (version: VersionInfo, clear: boolean) => {
    setMessageSaving(true);
    try {
      const updated = await setVersionMessage(
        object.id,
        version.id,
        clear ? null : messageDraft,
      );
      setVersions((prev) =>
        prev.map((v) => (v.id === updated.id ? updated : v)),
      );
      onObjectUpdated();
      setEditingVersionId(null);
      toast.success("Nachricht aktualisiert");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Nachricht konnte nicht geändert werden",
      );
    } finally {
      setMessageSaving(false);
    }
  };

  const handleStartDiff = () => {
    if (diffSelection.length === 2) {
      const sorted = [...diffSelection].sort((a, b) => a.number - b.number);
      onOpenDiff(sorted[0], sorted[1]);
      setDiffSelectMode(false);
      setDiffSelection([]);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Clock className="w-4 h-4" />
            Versionshistorie: {object.name}
          </DialogTitle>
        </DialogHeader>

        <div className="flex items-center gap-2 pb-2 border-b border-border/50">
          {!object.isSealed && (
            <Button
              size="sm"
              variant="outline"
              onClick={() => onOpenEditor(object)}
            >
              <Pencil className="w-3.5 h-3.5 mr-1.5" />
              Neue Version
            </Button>
          )}
          <Button
            size="sm"
            variant={diffSelectMode ? "default" : "outline"}
            onClick={() => {
              if (diffSelectMode && diffSelection.length === 2) {
                handleStartDiff();
              } else {
                setDiffSelectMode(!diffSelectMode);
                setDiffSelection([]);
              }
            }}
            disabled={versions.length < 2}
          >
            <GitCompareArrows className="w-3.5 h-3.5 mr-1.5" />
            {diffSelectMode
              ? diffSelection.length === 2
                ? "Vergleichen"
                : `${diffSelection.length}/2 gewählt`
              : "Vergleichen"}
          </Button>
          {diffSelectMode && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setDiffSelectMode(false);
                setDiffSelection([]);
              }}
            >
              Abbrechen
            </Button>
          )}
          {object.isSealed && (
            <Badge variant="destructive" className="ml-auto">
              <Lock className="w-3 h-3 mr-1" />
              Versiegelt
            </Badge>
          )}
        </div>

        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="flex items-center justify-center py-8 text-muted-foreground text-sm">
              Versionen werden geladen...
            </div>
          ) : versions.length === 0 ? (
            <div className="flex items-center justify-center py-8 text-muted-foreground text-sm">
              Keine Versionen gefunden.
            </div>
          ) : (
            <div className="space-y-1">
              {[...versions].reverse().map((version) => {
                const isDiffSelected = diffSelection.some(
                  (v) => v.id === version.id,
                );
                const validTransitions = VERSION_STATE_TRANSITIONS[
                  version.state as VersionState
                ]?.filter((s) => s !== version.state) ?? [];
                const isEditing = editingVersionId === version.id;
                const messageLabel =
                  version.commitMessage === null || version.commitMessage === undefined
                    ? "Keine Nachricht"
                    : version.commitMessage === ""
                      ? "Leere Nachricht"
                      : version.commitMessage;

                return (
                  <div
                    key={version.id}
                    className={cn(
                      "rounded-md border border-border/50 px-3 py-2.5 transition-colors",
                      version.isCurrent && "bg-primary/5 border-primary/20",
                      diffSelectMode && "cursor-pointer hover:bg-muted/60",
                      isDiffSelected && "bg-primary/10 border-primary/40",
                    )}
                    onClick={
                      diffSelectMode
                        ? () => handleDiffSelect(version)
                        : undefined
                    }
                  >
                    <div className="flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <span className="text-xs font-mono font-bold text-foreground/75 shrink-0">
                          v{version.number}
                        </span>
                        <Badge
                          variant={stateVariant(version.state)}
                          className="text-[10px] shrink-0"
                        >
                          {VERSION_STATE_LABELS[version.state as VersionState] ??
                            version.state}
                        </Badge>
                        {version.isCurrent && (
                          <Badge variant="secondary" className="text-[10px] shrink-0">
                            Aktuell
                          </Badge>
                        )}
                      </div>
                      {!diffSelectMode && (
                        <div className="flex items-center gap-1 shrink-0">
                          {validTransitions.length > 0 && (
                            <Select
                              onValueChange={(val) =>
                                handleStateChange(
                                  version,
                                  val as VersionState,
                                )
                              }
                            >
                              <SelectTrigger className="h-7 w-auto text-[11px] gap-1 px-2">
                                <SelectValue placeholder="Status" />
                              </SelectTrigger>
                              <SelectContent>
                                {validTransitions.map((s) => (
                                  <SelectItem
                                    key={s}
                                    value={s}
                                    className="text-xs"
                                  >
                                    {VERSION_STATE_LABELS[s]}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          )}
                          {!version.isCurrent && (
                            <Button
                              size="icon-xs"
                              variant="ghost"
                              className="h-7 w-7"
                              title="Version aktivieren"
                              onClick={() => handleCheckout(version)}
                            >
                              <RotateCcw className="w-3.5 h-3.5" />
                            </Button>
                          )}
                          <Button
                            size="icon-xs"
                            variant="ghost"
                            className="h-7 w-7"
                            title="Version exportieren"
                            onClick={() => handleExport(version)}
                          >
                            <ArrowDownToLine className="w-3.5 h-3.5" />
                          </Button>
                          <Button
                            size="icon-xs"
                            variant="ghost"
                            className="h-7 w-7"
                            title="Nachricht bearbeiten"
                            onClick={() => startMessageEdit(version)}
                          >
                            <MessageSquareText className="w-3.5 h-3.5" />
                          </Button>
                        </div>
                      )}
                    </div>
                    <div className="mt-1 flex items-center gap-3 text-[11px] text-muted-foreground">
                      <span>{formatDate(version.createdAt)}</span>
                      <span>{formatBytes(version.sizeBytes)}</span>
                    </div>
                    {isEditing ? (
                      <div className="mt-2 space-y-2">
                        <Textarea
                          value={messageDraft}
                          onChange={(event) => setMessageDraft(event.target.value)}
                          placeholder="Neue Nachricht (optional)"
                          className="text-xs"
                          rows={3}
                        />
                        <div className="flex flex-wrap items-center gap-2">
                          <Button
                            size="sm"
                            variant="default"
                            onClick={() => handleMessageSave(version, false)}
                            disabled={messageSaving}
                          >
                            <Check className="w-3.5 h-3.5 mr-1" />
                            Speichern
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => handleMessageSave(version, true)}
                            disabled={messageSaving}
                          >
                            Nachricht löschen
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={cancelMessageEdit}
                            disabled={messageSaving}
                          >
                            <X className="w-3.5 h-3.5 mr-1" />
                            Abbrechen
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <p className="mt-1 text-xs text-foreground/75 italic truncate">
                        {messageLabel}
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};
