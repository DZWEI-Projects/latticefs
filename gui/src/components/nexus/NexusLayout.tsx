import { useEffect, useMemo, useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Sidebar } from "./Sidebar";
import { Toolbar } from "./Toolbar";
import { ContentArea } from "./ContentArea";
import { StatusBar } from "./StatusBar";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { addObjectTag, removeObjectTag, setObjectTrustLevel, openObject } from "@/lib/lfs";
import { useViewObjects } from "@/hooks/useViewObjects";
import { toast } from "sonner";
import { ObjectDetailPanel } from "./ObjectDetailPanel";
import { TagDialog } from "./TagDialog";
import { NexusSettingsDialog } from "./NexusSettingsDialog";

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
  currentViewName?: string;
  onViewChange: (viewName: string) => void;
}

export const NexusLayout = ({ currentViewName, onViewChange }: NexusLayoutProps) => {
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
  const [tagDialogOpen, setTagDialogOpen] = useState(false);
  const [tagTargetId, setTagTargetId] = useState<string | null>(null);

  const { data, isLoading, error } = useViewObjects(currentViewName || "recent");
  const [objects, setObjects] = useState<ObjectInfo[]>([]);

  useEffect(() => {
    if (data) {
      setObjects(data);
    }
  }, [data]);

  useEffect(() => {
    setSelectedObjects([]);
    setActiveObjectId(null);
  }, [currentViewName]);

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
      .then(() => toast.success(`Opened ${object.name}`))
      .catch((err) => toast.error(err?.message || "Failed to open file"));
  }, []);

  const handleRequestAddTag = useCallback((object: ObjectInfo) => {
    setTagTargetId(object.id);
    setTagDialogOpen(true);
  }, []);

  const handleAddTag = useCallback(
    async (tag: TagInfo) => {
      if (!tagTargetId) return;
      try {
        const updated = await addObjectTag(tagTargetId, tag);
        setObjects((prev) =>
          prev.map((obj) =>
            obj.id === tagTargetId
              ? updated ?? {
                  ...obj,
                  tags: obj.tags.some(
                    (existing) =>
                      existing.key === tag.key && existing.value === tag.value
                  )
                    ? obj.tags
                    : [...obj.tags, tag],
                }
              : obj
          )
        );
        toast.success("Tag added");
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Failed to add tag");
      }
    },
    [tagTargetId]
  );

  const handleRemoveTag = useCallback(async (object: ObjectInfo, tag: TagInfo) => {
    try {
      const updated = await removeObjectTag(object.id, tag);
      setObjects((prev) =>
        prev.map((obj) =>
          obj.id === object.id
            ? updated ?? {
                ...obj,
                tags: obj.tags.filter(
                  (existing) =>
                    !(existing.key === tag.key && existing.value === tag.value)
                ),
              }
            : obj
        )
      );
      toast.success("Tag removed");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to remove tag");
    }
  }, []);

  const handleSetTrust = useCallback(async (object: ObjectInfo, trust: number | null) => {
    try {
      const updated = await setObjectTrustLevel(object.id, trust);
      setObjects((prev) =>
        prev.map((obj) =>
          obj.id === object.id
            ? updated ?? { ...obj, trustLevel: trust }
            : obj
        )
      );
      toast.success("Trust score updated");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update trust");
    }
  }, []);

  const handleShowDetails = useCallback((object: ObjectInfo) => {
    setActiveObjectId(object.id);
    setSelectedObjects([object.id]);
    setDetailPanelOpen(true);
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
          LatticeFS
        </span>
      </div>

      {/* Main content area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <Sidebar
          currentViewName={currentViewName}
          onViewSelect={onViewChange}
          onOpenSettings={() => setSettingsOpen(true)}
        />

        {/* Main panel */}
        <div className="flex flex-col flex-1 overflow-hidden">
          {/* Toolbar */}
          <Toolbar
            currentViewName={currentViewName}
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
              sort={sort}
              onSortChange={setSort}
              filters={filters}
              searchQuery={searchQuery}
            />
            {activeObject && detailPanelOpen && (
              <ObjectDetailPanel
                object={activeObject}
                onClose={() => setDetailPanelOpen(false)}
                onRequestAddTag={handleRequestAddTag}
                onRemoveTag={handleRemoveTag}
                onSetTrust={handleSetTrust}
              />
            )}
          </div>

          {/* Status bar */}
          <StatusBar
            viewName={currentViewName}
            selectedCount={selectedObjects.length}
          />
        </div>
      </div>

      <TagDialog
        open={tagDialogOpen}
        onOpenChange={(open) => {
          setTagDialogOpen(open);
          if (!open) {
            setTagTargetId(null);
          }
        }}
        onSubmit={handleAddTag}
      />

      <NexusSettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        showDetailsOnSelect={showDetailsOnSelect}
        onShowDetailsOnSelectChange={(value) => {
          setShowDetailsOnSelect(value);
          localStorage.setItem("nexus-details-on-select", String(value));
        }}
      />
    </div>
  );
};
