import { useEffect, useMemo, useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Sidebar } from "./Sidebar";
import { Toolbar } from "./Toolbar";
import { ContentArea } from "./ContentArea";
import { StatusBar } from "./StatusBar";
import type { ObjectInfo, TagInfo, VersionInfo } from "@/lib/lfs";
import { addObjectTag, removeObjectTag, setObjectTrustLevel, openObject, isTextEditable } from "@/lib/lfs";
import { useViewObjects } from "@/hooks/useViewObjects";
import { useViews } from "@/hooks/useViews";
import { toast } from "sonner";
import { ObjectDetailPanel } from "./ObjectDetailPanel";
import { NexusSettingsDialog } from "./NexusSettingsDialog";
import { VersionHistoryDialog } from "./VersionHistoryDialog";
import { VersionDiffDialog } from "./VersionDiffDialog";
import { TextEditorDialog } from "./TextEditorDialog";
import { ObjectRenameDialog } from "./ObjectRenameDialog";

export type ViewMode = "graph" | "grid" | "list";
export type SortField =
  | "name"
  | "extension"
  | "sizeBytes"
  | "modifiedAt"
  | "createdAt"
  | "trustLevel";
export type SortDirection = "asc" | "desc";
export interface SortState {
  field: SortField;
  direction: SortDirection;
}

export interface FilterState {
  type: "all" | "document" | "image" | "video" | "audio" | "code" | "archive" | "other";
  trustMin: number | null;
  tag: string;
  onlyTagged: boolean;
}

interface NexusLayoutProps {
  currentViewId?: string;
  onViewChange: (viewId: string) => void;
}

export const NexusLayout = ({ currentViewId, onViewChange }: NexusLayoutProps) => {
  const [viewMode, setViewMode] = useState<ViewMode>(() => {
    // Check localStorage for saved preference, default to graph
    const saved = localStorage.getItem("nexus-view-mode");
    return (saved as ViewMode) || "graph";
  });
  const [selectedObjects, setSelectedObjects] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [sort, setSort] = useState<SortState>({ field: "name", direction: "asc" });
  const [filters, setFilters] = useState<FilterState>({
    type: "all",
    trustMin: null,
    tag: "",
    onlyTagged: false,
  });
  const [activeObjectId, setActiveObjectId] = useState<string | null>(null);
  const [detailPanelOpen, setDetailPanelOpen] = useState(true);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [showDetailsOnSelect, setShowDetailsOnSelect] = useState(() => {
    const saved = localStorage.getItem("nexus-details-on-select");
    return saved ? saved === "true" : true;
  });

  // Version management dialog state
  const [versionHistoryObject, setVersionHistoryObject] = useState<ObjectInfo | null>(null);
  const [editorObject, setEditorObject] = useState<ObjectInfo | null>(null);
  const [diffObject, setDiffObject] = useState<ObjectInfo | null>(null);
  const [diffVersionA, setDiffVersionA] = useState<VersionInfo | null>(null);
  const [diffVersionB, setDiffVersionB] = useState<VersionInfo | null>(null);
  const [renameObject, setRenameObject] = useState<ObjectInfo | null>(null);

  const { data: views = [] } = useViews();
  const { data, isLoading, error } = useViewObjects(currentViewId || "recent");
  const [objects, setObjects] = useState<ObjectInfo[]>([]);

  useEffect(() => {
    if (data) {
      setObjects(data);
    }
  }, [data]);

  useEffect(() => {
    setSelectedObjects([]);
    setActiveObjectId(null);
  }, [currentViewId]);

  const activeObject = useMemo(
    () => objects.find((obj) => obj.id === activeObjectId) || null,
    [activeObjectId, objects]
  );

  const handleViewModeChange = useCallback((mode: ViewMode) => {
    setViewMode(mode);
    localStorage.setItem("nexus-view-mode", mode);
  }, []);

  const handleObjectSelect = useCallback((objectId: string, multiSelect?: boolean) => {
    setSelectedObjects((prev) => {
      if (multiSelect) {
        return prev.includes(objectId)
          ? prev.filter((id) => id !== objectId)
          : [...prev, objectId];
      }
      return [objectId];
    });
    setActiveObjectId(objectId);
    if (showDetailsOnSelect) {
      setDetailPanelOpen(true);
    }
  }, [showDetailsOnSelect]);

  const handleObjectOpen = useCallback((object: ObjectInfo) => {
    openObject(object.id)
      .then(() => toast.success(`${object.name} geöffnet`))
      .catch((err) => toast.error(err?.message || "Datei konnte nicht geöffnet werden"));
  }, []);

  const handleRequestAddTag = useCallback((object: ObjectInfo) => {
    setActiveObjectId(object.id);
    setSelectedObjects([object.id]);
    setDetailPanelOpen(true);
  }, []);

  const handleAddTag = useCallback(async (object: ObjectInfo, tag: TagInfo) => {
    try {
      const updated = await addObjectTag(object.id, tag);
      setObjects((prev) =>
        prev.map((obj) => {
          if (obj.id !== object.id) return obj;
          if (updated) return { ...obj, ...updated, views: obj.views };
          const exists = obj.tags.some(
            (existing) => existing.key === tag.key && existing.value === tag.value
          );
          return exists ? obj : { ...obj, tags: [...obj.tags, tag] };
        })
      );
      toast.success("Eigenschaft hinzugefügt");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Eigenschaft konnte nicht hinzugefügt werden");
      throw err;
    }
  }, []);

  const handleRemoveTag = useCallback(async (object: ObjectInfo, tag: TagInfo) => {
    try {
      const updated = await removeObjectTag(object.id, tag);
      setObjects((prev) =>
        prev.map((obj) =>
          obj.id === object.id
            ? updated
              ? { ...obj, ...updated, views: obj.views }
              : {
                  ...obj,
                  tags: obj.tags.filter(
                    (existing) =>
                      !(existing.key === tag.key && existing.value === tag.value)
                  ),
                }
            : obj
        )
      );
      toast.success("Eigenschaft entfernt");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Eigenschaft konnte nicht entfernt werden");
    }
  }, []);

  const handleUpdateTag = useCallback(
    async (object: ObjectInfo, previous: TagInfo, next: TagInfo) => {
      if (previous.key === next.key && previous.value === next.value) {
        return;
      }
      try {
        await removeObjectTag(object.id, previous);
        const updated = await addObjectTag(object.id, next);
        setObjects((prev) =>
          prev.map((obj) => {
            if (obj.id !== object.id) return obj;
            if (updated) return { ...obj, ...updated, views: obj.views };
            const filtered = obj.tags.filter(
              (existing) =>
                !(existing.key === previous.key && existing.value === previous.value)
            );
            const exists = filtered.some(
              (existing) => existing.key === next.key && existing.value === next.value
            );
            return exists ? { ...obj, tags: filtered } : { ...obj, tags: [...filtered, next] };
          })
        );
        toast.success("Eigenschaft aktualisiert");
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Eigenschaft konnte nicht aktualisiert werden");
        throw err;
      }
    },
    []
  );

  const handleSetTrust = useCallback(async (object: ObjectInfo, trust: number | null) => {
    try {
      const updated = await setObjectTrustLevel(object.id, trust);
      setObjects((prev) =>
        prev.map((obj) =>
          obj.id === object.id
            ? updated
              ? { ...obj, ...updated, views: obj.views }
              : { ...obj, trustLevel: trust }
            : obj
        )
      );
      toast.success("Sicherheitsgrad aktualisiert");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Sicherheitsgrad konnte nicht aktualisiert werden");
    }
  }, []);

  const handleShowDetails = useCallback((object: ObjectInfo) => {
    setActiveObjectId(object.id);
    setSelectedObjects([object.id]);
    setDetailPanelOpen(true);
  }, []);

  const handleRequestRename = useCallback((object: ObjectInfo) => {
    setActiveObjectId(object.id);
    setSelectedObjects([object.id]);
    setRenameObject(object);
  }, []);

  const encodeBase64Url = (value: string) => {
    const bytes = new TextEncoder().encode(value);
    let binary = "";
    bytes.forEach((byte) => {
      binary += String.fromCharCode(byte);
    });
    return btoa(binary)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");
  };

  const handleRenameObject = useCallback(
    async (object: ObjectInfo, name: string) => {
      const trimmed = name.trim();
      if (!trimmed) {
        toast.error("Der Name darf nicht leer sein.");
        return;
      }

      const encoded = encodeBase64Url(trimmed);
      const previous = object.tags.find((tag) => tag.key === "auto:filename_b64");
      const nextTag: TagInfo = { key: "auto:filename_b64", value: encoded };

      try {
        if (previous) {
          await removeObjectTag(object.id, previous);
        }
        const updated = await addObjectTag(object.id, nextTag);
        setObjects((prev) =>
          prev.map((obj) => {
            if (obj.id !== object.id) return obj;
            if (updated) return { ...obj, ...updated, views: obj.views };
            const filtered = obj.tags.filter(
              (tag) => !(tag.key === "auto:filename_b64" && tag.value === previous?.value)
            );
            const exists = filtered.some(
              (tag) => tag.key === nextTag.key && tag.value === nextTag.value
            );
            return { ...obj, name: trimmed, tags: exists ? filtered : [...filtered, nextTag] };
          })
        );
        toast.success("Dateiname aktualisiert");
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Dateiname konnte nicht aktualisiert werden");
        throw err;
      }
    },
    []
  );

  const handleOpenVersions = useCallback((object: ObjectInfo) => {
    setVersionHistoryObject(object);
  }, []);

  const handleOpenEditor = useCallback((object: ObjectInfo) => {
    setEditorObject(object);
  }, []);

  const handleOpenDiff = useCallback((object: ObjectInfo, versionA: VersionInfo, versionB: VersionInfo) => {
    setDiffObject(object);
    setDiffVersionA(versionA);
    setDiffVersionB(versionB);
  }, []);

  const handleObjectUpdated = useCallback((updated?: ObjectInfo) => {
    if (updated) {
      setObjects((prev) =>
        prev.map((obj) => (obj.id === updated.id ? { ...obj, ...updated, views: obj.views } : obj)),
      );
    }
  }, []);

  const handleEditorObjectUpdated = useCallback((updated: ObjectInfo) => {
    setObjects((prev) =>
      prev.map((obj) => (obj.id === updated.id ? { ...obj, ...updated, views: obj.views } : obj)),
    );
    setEditorObject(updated);
  }, []);

  const handleDragStart = useCallback((e: React.MouseEvent) => {
    if (e.buttons === 1) {
      getCurrentWindow().startDragging();
    }
  }, []);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-background">
      {/* Title bar / drag region */}
      <div
        className="h-10 flex-shrink-0 flex items-center justify-center border-b border-border/50 select-none cursor-default"
        onMouseDown={handleDragStart}
      >
        <span className="text-xs font-medium text-muted-foreground tracking-wide">
          NeuralFS
        </span>
      </div>

      {/* Main content area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <Sidebar
          currentViewId={currentViewId}
          onViewSelect={onViewChange}
          onOpenSettings={() => setSettingsOpen(true)}
        />

        {/* Main panel */}
        <div className="flex flex-col flex-1 overflow-hidden">
          {/* Toolbar */}
          <Toolbar
            currentViewId={currentViewId}
            viewMode={viewMode}
            onViewModeChange={handleViewModeChange}
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
            sort={sort}
            onSortChange={setSort}
            filters={filters}
            onFiltersChange={setFilters}
          />

          {/* Content */}
          <div className="flex flex-1 overflow-hidden">
            <ContentArea
              viewMode={viewMode}
              objects={objects}
              isLoading={isLoading}
              error={error ?? null}
              selectedObjects={selectedObjects}
              onObjectSelect={handleObjectSelect}
              onObjectOpen={handleObjectOpen}
              onObjectFocus={(object) => {
                setActiveObjectId(object.id);
                if (showDetailsOnSelect) setDetailPanelOpen(true);
              }}
              onRequestAddTag={handleRequestAddTag}
              onRemoveTag={handleRemoveTag}
              onSetTrust={handleSetTrust}
              onShowDetails={handleShowDetails}
              onOpenVersions={handleOpenVersions}
              onOpenEditor={handleOpenEditor}
              onRenameObject={handleRequestRename}
              views={views}
              onViewSelect={onViewChange}
              sort={sort}
              onSortChange={setSort}
              filters={filters}
              searchQuery={searchQuery}
            />
            {activeObject && detailPanelOpen && (
              <ObjectDetailPanel
                object={activeObject}
                currentViewId={currentViewId}
                onClose={() => setDetailPanelOpen(false)}
                onAddTag={handleAddTag}
                onRemoveTag={handleRemoveTag}
                onUpdateTag={handleUpdateTag}
                onSetTrust={handleSetTrust}
                onViewSelect={onViewChange}
                onOpenVersions={handleOpenVersions}
                onOpenEditor={handleOpenEditor}
              />
            )}
          </div>

          {/* Status bar */}
          <StatusBar
            viewId={currentViewId}
            selectedCount={selectedObjects.length}
          />
        </div>
      </div>

      <NexusSettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        showDetailsOnSelect={showDetailsOnSelect}
        onShowDetailsOnSelectChange={(value) => {
          setShowDetailsOnSelect(value);
          localStorage.setItem("nexus-details-on-select", String(value));
        }}
      />

      {versionHistoryObject && (
        <VersionHistoryDialog
          open={!!versionHistoryObject}
          onOpenChange={(open) => { if (!open) setVersionHistoryObject(null); }}
          object={versionHistoryObject}
          onOpenDiff={(versionA, versionB) =>
            handleOpenDiff(versionHistoryObject, versionA, versionB)
          }
          onOpenEditor={(obj) => setEditorObject(obj)}
          onObjectUpdated={handleObjectUpdated}
        />
      )}

      {diffObject && (
        <VersionDiffDialog
          open={!!diffObject}
          onOpenChange={(open) => {
            if (!open) {
              setDiffObject(null);
              setDiffVersionA(null);
              setDiffVersionB(null);
            }
          }}
          object={diffObject}
          versionA={diffVersionA}
          versionB={diffVersionB}
        />
      )}

      {editorObject && (
        <TextEditorDialog
          open={!!editorObject}
          onOpenChange={(open) => { if (!open) setEditorObject(null); }}
          object={editorObject}
          onObjectUpdated={handleEditorObjectUpdated}
        />
      )}

      {renameObject && (
        <ObjectRenameDialog
          open={!!renameObject}
          onOpenChange={(open) => { if (!open) setRenameObject(null); }}
          object={renameObject}
          onRename={handleRenameObject}
        />
      )}
    </div>
  );
};
