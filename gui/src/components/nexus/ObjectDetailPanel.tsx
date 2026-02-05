import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { useViews } from "@/hooks/useViews";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { cn } from "@/lib/utils";
import { Check, ChevronDown, Pencil, Plus, Trash2, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { KeyboardEvent } from "react";
import {
  Popover,
  PopoverArrow,
  PopoverContent,
  PopoverTrigger,
} from "../ui/popover";
import { toast } from "sonner";

interface ObjectDetailPanelProps {
  object: ObjectInfo;
  currentViewId?: string;
  onClose: () => void;
  onAddTag: (object: ObjectInfo, tag: TagInfo) => Promise<void>;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onUpdateTag: (object: ObjectInfo, previous: TagInfo, next: TagInfo) => Promise<void>;
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

function isAutoTag(tag: TagInfo) {
  return tag.key.startsWith("auto:");
}

function isSystemTag(tag: TagInfo) {
  return tag.key.startsWith("sys:");
}

const ID3_PREFIX = "auto:id3:";
const EXIF_PREFIX = "auto:exif:";

function formatTagLabel(raw: string): string {
  if (!raw) return "—";
  const withSpaces = raw.replace(/_/g, " ");
  return withSpaces.charAt(0).toUpperCase() + withSpaces.slice(1);
}

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
}: ObjectDetailPanelProps) => {
  const [trustValue, setTrustValue] = useState<number>(object.trustLevel ?? 70);
  const [autoTagsOpen, setAutoTagsOpen] = useState(false);
  const { userTags, autoTags, systemTags, id3Tags, exifTags } = useMemo(() => {
    const user: TagInfo[] = [];
    const auto: TagInfo[] = [];
    const system: TagInfo[] = [];
    const id3: TagInfo[] = [];
    const exif: TagInfo[] = [];
    for (const tag of object.tags) {
      if (isAutoTag(tag)) {
        if (tag.key.startsWith(ID3_PREFIX)) {
          id3.push(tag);
        } else if (tag.key.startsWith(EXIF_PREFIX)) {
          exif.push(tag);
        } else {
          auto.push(tag);
        }
      } else if (isSystemTag(tag)) {
        system.push(tag);
      } else {
        user.push(tag);
      }
    }
    return {
      userTags: user,
      autoTags: auto,
      systemTags: system,
      id3Tags: id3,
      exifTags: exif,
    };
  }, [object.tags]);

  useEffect(() => {
    setTrustValue(object.trustLevel ?? 70);
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

  return (
    <aside className="w-80 lg:w-[400px] border-l border-border/50 bg-background/80 flex flex-col">
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
              <span className="text-foreground/75">Dateiendung</span>
              <span className="font-medium uppercase">
                {object.extension || "—"}
              </span>
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
                onClick={handleCopyId}
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
        />

        <section className="space-y-3">
          <Collapsible open={autoTagsOpen} onOpenChange={setAutoTagsOpen}>
            <div className="flex items-center justify-between">
              <CollapsibleTrigger className="flex items-center gap-1.5 text-xs font-semibold text-foreground/75 uppercase tracking-wider hover:text-foreground transition-colors">
                <ChevronDown
                  className={cn(
                    "w-3 h-3 transition-transform",
                    !autoTagsOpen && "-rotate-90",
                  )}
                />
                Automatische Tags
              </CollapsibleTrigger>
              <span className="text-xs text-muted-foreground">
                {autoTags.length}
              </span>
            </div>
            <CollapsibleContent className="pt-2">
              <ReadOnlyTagsList
                tags={autoTags}
                emptyLabel="Keine automatischen Tags erkannt."
                badgeLabel="AUTO"
              />
            </CollapsibleContent>
          </Collapsible>
        </section>

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
                      className="h-8 text-xs p-0! border-none! focus-visible:ring-0! focus-visible:ring-offset-0!"
                      disabled={isSaving}
                    />
                    <Input
                      value={editValue}
                      onChange={(e) => setEditValue(e.target.value)}
                      onKeyDown={handleEditKeyDown}
                      className="h-8 text-xs p-0! border-none! focus-visible:ring-0! focus-visible:ring-offset-0!"
                      disabled={isSaving}
                    />
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 p-0! border-none! focus-visible:ring-0! focus-visible:ring-offset-0!"
                      onClick={handleSaveEdit}
                      disabled={isSaving}
                      aria-label="Eigenschaft speichern"
                    >
                      <Check className="w-4 h-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 p-0! border-none! focus-visible:ring-0! focus-visible:ring-offset-0!"
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

function TagDetailsSection({
  title,
  tags,
  prefix,
}: {
  title: string;
  tags: TagInfo[];
  prefix: string;
}) {
  const [open, setOpen] = useState(false);
  const details = useMemo(
    () =>
      tags
        .map((tag) => ({
          key: `${tag.key}:${tag.value}`,
          label: formatTagLabel(tag.key.replace(prefix, "")),
          value: tag.value,
        }))
        .sort((a, b) =>
          a.label.localeCompare(b.label, undefined, { sensitivity: "base", numeric: true }),
        ),
    [tags, prefix],
  );

  if (tags.length === 0) return null;

  return (
    <section className="space-y-3">
      <Collapsible open={open} onOpenChange={setOpen}>
        <div className="flex items-center justify-between">
          <CollapsibleTrigger className="flex items-center gap-1.5 text-xs font-semibold text-foreground/75 uppercase tracking-wider hover:text-foreground transition-colors">
            <ChevronDown
              className={cn(
                "w-3 h-3 transition-transform",
                !open && "-rotate-90",
              )}
            />
            {title}
          </CollapsibleTrigger>
          <span className="text-xs text-muted-foreground">{tags.length}</span>
        </div>
        <CollapsibleContent className="pt-2">
          <div className="space-y-2 text-xs">
            {details.map((detail) => (
              <div
                key={detail.key}
                className="flex items-center justify-between gap-2"
              >
                <span className="text-foreground/75">{detail.label}</span>
                <span
                  className="font-medium text-right truncate max-w-[180px]"
                  title={detail.value}
                >
                  {detail.value}
                </span>
              </div>
            ))}
          </div>
        </CollapsibleContent>
      </Collapsible>
    </section>
  );
}

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
