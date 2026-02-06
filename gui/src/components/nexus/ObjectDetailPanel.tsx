import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { isTextEditable, exportObjectVersion } from "@/lib/lfs";
import { cn } from "@/lib/utils";
import {
  ArrowDownToLine,
  Check,
  Clock,
  Pencil,
  Plus,
  Trash2,
  X,
  AlertTriangle,
} from "lucide-react";
import { useEffect, useMemo, useState, useRef } from "react";
import type { KeyboardEvent } from "react";
import { toast } from "sonner";
import { ObjectInfoSection } from "@/components/nexus/ObjectInfoSection";
import { TagDetailsSection } from "@/components/nexus/TagDetailsSection";
import { formatExifFieldLabel } from "@/lib/metadataDisplay";
import { DEFAULT_TRUST_LEVEL, QUARANTINE_TRUST_LEVEL } from "@/lib/trustConstants";

interface ObjectDetailPanelProps {
  object: ObjectInfo;
  currentViewId?: string;
  onClose: () => void;
  onAddTag: (object: ObjectInfo, tag: TagInfo) => Promise<void>;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onUpdateTag: (object: ObjectInfo, previous: TagInfo, next: TagInfo) => Promise<void>;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  onViewSelect: (viewId: string) => void;
  onOpenVersions?: (object: ObjectInfo) => void;
  onOpenEditor?: (object: ObjectInfo) => void;
}

function isAutoTag(tag: TagInfo) {
  return tag.key.startsWith("auto:");
}

function isSystemTag(tag: TagInfo) {
  return tag.key.startsWith("sys:");
}

const ID3_PREFIX = "auto:id3:";
const EXIF_PREFIX = "auto:exif:";

function parseTagInput(keyInput: string, valueInput: string): TagInfo | null {
  const key = keyInput.trim();
  const value = valueInput.trim();
  if (key && value) return { key, value };
  const combined = key || value;
  if (!combined) return null;
  const match = combined.match(/^([^:=]+)\s*[:=]\s*(.+)$/);
  if (match) {
    return { key: match[1].trim(), value: match[2].trim() };
  }
  const parts = combined.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return { key: parts[0].trim(), value: parts.slice(1).join(" ").trim() };
  }
  return null;
}

export const ObjectDetailPanel = ({
  object,
  currentViewId,
  onClose,
  onAddTag,
  onRemoveTag,
  onUpdateTag,
  onSetTrust,
  onViewSelect,
  onOpenVersions,
  onOpenEditor,
}: ObjectDetailPanelProps) => {
  const [trustValue, setTrustValue] = useState<number>(object.trustLevel ?? DEFAULT_TRUST_LEVEL);
  const priorTrustRef = useRef<number | null>(null);
  const { userTags, systemTags, id3Tags, exifTags, mimeType } = useMemo(() => {
    const user: TagInfo[] = [];
    const system: TagInfo[] = [];
    const id3: TagInfo[] = [];
    const exif: TagInfo[] = [];
    let detectedMimeType: string | null = null;
    for (const tag of object.tags) {
      if (isAutoTag(tag)) {
        if (tag.key === "auto:mimetype" && !detectedMimeType) {
          detectedMimeType = tag.value;
          continue;
        }
        if (tag.key.startsWith(ID3_PREFIX)) {
          id3.push(tag);
          continue;
        }
        if (tag.key.startsWith(EXIF_PREFIX)) {
          exif.push(tag);
        }
      } else if (isSystemTag(tag)) {
        system.push(tag);
      } else {
        user.push(tag);
      }
    }
    return {
      userTags: user,
      systemTags: system,
      id3Tags: id3,
      exifTags: exif,
      mimeType: detectedMimeType,
    };
  }, [object.tags]);

  useEffect(() => {
    setTrustValue(object.trustLevel ?? DEFAULT_TRUST_LEVEL);
    priorTrustRef.current = null;
  }, [object.id, object.trustLevel]);

  const handleCopyId = async () => {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(object.id);
      } else {
        const textarea = document.createElement("textarea");
        textarea.value = object.id;
        textarea.style.position = "fixed";
        textarea.style.opacity = "0";
        document.body.appendChild(textarea);
        textarea.focus();
        textarea.select();
        document.execCommand("copy");
        document.body.removeChild(textarea);
      }
      toast.success("Objekt-ID kopiert");
    } catch {
      toast.error("Objekt-ID konnte nicht kopiert werden");
    }
  };

  const clampedTrust = useMemo(
    () => Math.max(0, Math.min(100, trustValue)),
    [trustValue],
  );
  const isQuarantined = clampedTrust === QUARANTINE_TRUST_LEVEL;

  return (
    <aside className="w-80 xl:w-[400px] 2xl:w-[470px] border-l border-border/50 bg-background/80 flex flex-col">
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
        <ObjectInfoSection
          object={object}
          currentViewId={currentViewId}
          mimeType={mimeType}
          onCopyId={handleCopyId}
          onViewSelect={onViewSelect}
          onOpenVersions={() => onOpenVersions?.(object)}
        />

        {/* Version actions */}
        <section className="space-y-2">
          <div className="flex flex-wrap gap-1.5">
            {object.versionCount > 1 && (
              <Button
                size="sm"
                variant="outline"
                className="h-7 text-xs"
                onClick={() => onOpenVersions?.(object)}
              >
                <Clock className="w-3 h-3 mr-1" />
                {object.versionCount} Versionen
              </Button>
            )}
            {isTextEditable(object.extension) && !object.isSealed && (
              <Button
                size="sm"
                variant="outline"
                className="h-7 text-xs"
                onClick={() => onOpenEditor?.(object)}
              >
                <Pencil className="w-3 h-3 mr-1" />
                Bearbeiten
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              className="h-7 text-xs"
              onClick={async () => {
                try {
                  await exportObjectVersion(object.id);
                  toast.success("Exportiert");
                } catch (err) {
                  toast.error(err instanceof Error ? err.message : "Export fehlgeschlagen");
                }
              }}
            >
              <ArrowDownToLine className="w-3 h-3 mr-1" />
              Exportieren
            </Button>
          </div>
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
              Eigenschaften
            </h3>
            <span className="text-xs text-muted-foreground">
              {userTags.length}
            </span>
          </div>
          <TagsEditor
            object={object}
            tags={userTags}
            onAddTag={onAddTag}
            onRemoveTag={onRemoveTag}
            onUpdateTag={onUpdateTag}
          />
        </section>

        <TagDetailsSection
          title="Musikdetails"
          tags={id3Tags}
          prefix={ID3_PREFIX}
        />

        <TagDetailsSection
          title="Bildmetadaten (EXIF)"
          tags={exifTags}
          prefix={EXIF_PREFIX}
          labelFormatter={formatExifFieldLabel}
        />

        {systemTags.length > 0 && (
          <section className="space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
                System-Tags
              </h3>
              <span className="text-xs text-muted-foreground">
                {systemTags.length}
              </span>
            </div>
            <ReadOnlyTagsList
              tags={systemTags}
              emptyLabel="Keine System-Tags."
              badgeLabel="SYS"
            />
          </section>
        )}

        <section className="space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
              Quarantäne
            </h3>
            <Badge variant={isQuarantined ? "destructive" : "outline"} className="text-[10px] uppercase">
              {isQuarantined ? "Aktiv" : "Inaktiv"}
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground">
            Markiert verdächtige Objekte als unsicher, sodass ausführbare Dateien blockiert bleiben.
          </p>
          <Button
            size="sm"
            variant="outline"
            className="h-7 text-xs"
            onClick={() => {
              let nextTrust: number;
              if (isQuarantined) {
                // Restore prior trust level or use default
                nextTrust = priorTrustRef.current ?? DEFAULT_TRUST_LEVEL;
                priorTrustRef.current = null;
              } else {
                // Save current committed trust level before quarantining
                priorTrustRef.current = object.trustLevel ?? DEFAULT_TRUST_LEVEL;
                nextTrust = QUARANTINE_TRUST_LEVEL;
              }
              setTrustValue(nextTrust);
              onSetTrust(object, nextTrust);
            }}
          >
            <AlertTriangle className="w-3 h-3 mr-1" />
            {isQuarantined ? "Quarantäne aufheben" : "In Quarantäne verschieben"}
          </Button>
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
              {isQuarantined ? "Quarantäne" : `${clampedTrust}%`}
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

function TagsEditor({
  object,
  tags,
  onAddTag,
  onRemoveTag,
  onUpdateTag,
}: {
  object: ObjectInfo;
  tags: TagInfo[];
  onAddTag: (object: ObjectInfo, tag: TagInfo) => Promise<void>;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onUpdateTag: (object: ObjectInfo, previous: TagInfo, next: TagInfo) => Promise<void>;
}) {
  const [draftKey, setDraftKey] = useState("");
  const [draftValue, setDraftValue] = useState("");
  const [addError, setAddError] = useState<string | null>(null);
  const [isAdding, setIsAdding] = useState(false);
  const [editingTag, setEditingTag] = useState<TagInfo | null>(null);
  const [editKey, setEditKey] = useState("");
  const [editValue, setEditValue] = useState("");
  const [editError, setEditError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    setDraftKey("");
    setDraftValue("");
    setAddError(null);
    setEditingTag(null);
    setEditError(null);
  }, [object.id]);

  const handleAdd = async () => {
    const parsed = parseTagInput(draftKey, draftValue);
    if (!parsed) {
      setAddError("Bitte Gruppe und Wert angeben.");
      return;
    }
    setIsAdding(true);
    setAddError(null);
    try {
      await onAddTag(object, parsed);
      setDraftKey("");
      setDraftValue("");
    } catch {
      // Error handling is surfaced via toast in the parent.
    } finally {
      setIsAdding(false);
    }
  };

  const startEdit = (tag: TagInfo) => {
    setEditingTag(tag);
    setEditKey(tag.key);
    setEditValue(tag.value);
    setEditError(null);
  };

  const cancelEdit = () => {
    setEditingTag(null);
    setEditError(null);
  };

  const handleSaveEdit = async () => {
    if (!editingTag) return;
    const parsed = parseTagInput(editKey, editValue);
    if (!parsed) {
      setEditError("Bitte Gruppe und Wert angeben.");
      return;
    }
    setIsSaving(true);
    setEditError(null);
    try {
      await onUpdateTag(object, editingTag, parsed);
      setEditingTag(null);
    } catch {
      // Error handling is surfaced via toast in the parent.
    } finally {
      setIsSaving(false);
    }
  };

  const handleAddKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      event.preventDefault();
      handleAdd();
    }
  };

  const handleEditKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      event.preventDefault();
      handleSaveEdit();
    }
  };

  return (
    <div className="space-y-2">
      <div className="rounded-md border border-dashed border-border/60 bg-muted/10 p-2 space-y-2">
        <div className="flex items-center gap-2">
          <Input
            placeholder="Gruppe (z. B. Projekt)"
            value={draftKey}
            onChange={(e) => setDraftKey(e.target.value)}
            onKeyDown={handleAddKeyDown}
            className="h-7"
            disabled={isAdding}
          />
          <Input
            placeholder="Wert (z. B. Phoenix)"
            value={draftValue}
            onChange={(e) => setDraftValue(e.target.value)}
            onKeyDown={handleAddKeyDown}
            className="h-7"
            disabled={isAdding}
          />
          <Button
            size="icon-xs"
            onClick={handleAdd}
            disabled={isAdding}
          >
            <Plus className="w-3.5 h-3.5" />
          </Button>
        </div>
        <div className="flex flex-col items-start text-[10px] text-muted-foreground">
          <span>Eigenschaften beeinflussen die Sichtbarkeit des Objekts in Perspektiven.</span>
          {addError && <span className="text-destructive">{addError}</span>}
        </div>
      </div>

      {tags.length === 0 ? (
        <p className="text-xs text-foreground/75">
          Noch keine Eigenschaften zugewiesen.
        </p>
      ) : (
        <div className="space-y-1">
          {tags.map((tag) => {
            const isEditing =
              editingTag?.key === tag.key && editingTag?.value === tag.value;
            if (isEditing) {
              return (
                <div
                  key={`${tag.key}:${tag.value}`}
                  className="rounded-md border border-border/50 bg-background/60 px-2 py-2 space-y-2"
                >
                  <div className="flex items-center gap-2">
                    <Input
                      value={editKey}
                      onChange={(e) => setEditKey(e.target.value)}
                      onKeyDown={handleEditKeyDown}
                      className="h-8 text-xs !p-0 !border-none !focus-visible:ring-0 !focus-visible:ring-offset-0"
                      disabled={isSaving}
                    />
                    <Input
                      value={editValue}
                      onChange={(e) => setEditValue(e.target.value)}
                      onKeyDown={handleEditKeyDown}
                      className="h-8 text-xs !p-0 !border-none !focus-visible:ring-0 !focus-visible:ring-offset-0"
                      disabled={isSaving}
                    />
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 !p-0 !border-none !focus-visible:ring-0 !focus-visible:ring-offset-0"
                      onClick={handleSaveEdit}
                      disabled={isSaving}
                      aria-label="Eigenschaft speichern"
                    >
                      <Check className="w-4 h-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 !p-0 !border-none !focus-visible:ring-0 !focus-visible:ring-offset-0"
                      onClick={cancelEdit}
                      disabled={isSaving}
                      aria-label="Bearbeitung abbrechen"
                    >
                      <X className="w-4 h-4" />
                    </Button>
                  </div>
                  {editError && (
                    <p className="text-[10px] text-destructive">{editError}</p>
                  )}
                </div>
              );
            }

            return (
              <div
                key={`${tag.key}:${tag.value}`}
                className="flex items-center gap-2 rounded-md border border-border/50 bg-background/60 px-2 py-1.5"
              >
                <span className="w-20 text-[10px] uppercase truncate">
                  {tag.key}
                </span>
                <span className="flex-1 text-xs truncate font-bold">{tag.value}</span>
                <Button
                  size="icon"
                  variant="ghost"
                  className="size-6 text-muted-foreground hover:text-foreground"
                  onClick={() => startEdit(tag)}
                  aria-label={`Eigenschaft bearbeiten ${tag.key}:${tag.value}`}
                >
                  <Pencil />
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  className="size-6 text-muted-foreground hover:text-destructive"
                  onClick={() => onRemoveTag(object, tag)}
                  aria-label={`Eigenschaft entfernen ${tag.key}:${tag.value}`}
                >
                  <Trash2 />
                </Button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function ReadOnlyTagsList({
  tags,
  emptyLabel,
  badgeLabel,
}: {
  tags: TagInfo[];
  emptyLabel: string;
  badgeLabel: string;
}) {
  if (tags.length === 0) {
    return <p className="text-xs text-foreground/75">{emptyLabel}</p>;
  }

  return (
    <div className="space-y-1">
      {tags.map((tag) => (
        <div
          key={`${tag.key}:${tag.value}`}
          className="flex items-center gap-2 rounded-md border border-border/50 bg-background/60 px-2 py-1.5"
        >
          <span className="w-24 text-[10px] uppercase truncate">
            {tag.key}
          </span>
          <span className="flex-1 text-xs truncate font-semibold">
            {tag.value}
          </span>
          <Badge variant="outline" className="text-[10px] uppercase">
            {badgeLabel}
          </Badge>
        </div>
      ))}
    </div>
  );
}
