import { useMemo } from "react";
import type { ObjectInfo } from "@/lib/lfs";
import { FolderOpen, Loader2 } from "lucide-react";
import { GraphView } from "./GraphView";
import { GridView } from "./GridView";
import { ListView } from "./ListView";
import type { FilterState, SortState, ViewMode } from "./NexusLayout";
import type { TagInfo } from "@/lib/lfs";

interface ContentAreaProps {
  viewMode: ViewMode;
  objects: ObjectInfo[];
  isLoading: boolean;
  error: Error | null;
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
  onObjectFocus: (object: ObjectInfo) => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  onShowDetails: (object: ObjectInfo) => void;
  sort: SortState;
  onSortChange: (next: SortState) => void;
  filters: FilterState;
  searchQuery?: string;
}

export const ContentArea = ({
  viewMode,
  objects,
  isLoading,
  error,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
  onObjectFocus,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  onShowDetails,
  sort,
  onSortChange,
  filters,
  searchQuery,
}: ContentAreaProps) => {
  const filteredObjects = useMemo(() => {
    const query = searchQuery?.toLowerCase().trim() || "";
    const tagQuery = filters.tag.toLowerCase().trim();

    const matchesType = (obj: ObjectInfo) => {
      if (filters.type === "all") return true;
      const ext = obj.extension?.toLowerCase() || "";
      const typeMap: Record<FilterState["type"], string[]> = {
        all: [],
        document: ["pdf", "doc", "docx", "txt", "md", "rtf"],
        image: ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp"],
        video: ["mp4", "mov", "avi", "mkv", "webm"],
        audio: ["mp3", "wav", "flac", "aac", "ogg"],
        code: ["py", "js", "ts", "tsx", "jsx", "rs", "go", "java", "cpp", "c", "h"],
        archive: ["zip", "rar", "7z", "tar", "gz"],
        other: [],
      };

      if (filters.type === "other") {
        const knownExtensions = Object.values(typeMap)
          .flat()
          .filter((value) => value.length > 0);
        return ext.length > 0 && !knownExtensions.includes(ext);
      }

      return typeMap[filters.type].includes(ext);
    };

    const matchesTrust = (obj: ObjectInfo) => {
      if (filters.trustMin === null) return true;
      return (obj.trustLevel ?? 0) >= filters.trustMin;
    };

    const matchesTags = (obj: ObjectInfo) => {
      if (filters.onlyTagged && obj.tags.length === 0) return false;
      if (!tagQuery) return true;
      return obj.tags.some(
        (tag) =>
          tag.key.toLowerCase().includes(tagQuery) ||
          tag.value.toLowerCase().includes(tagQuery)
      );
    };

    return objects
      .filter((obj) => {
        if (!query) return true;
        return (
          obj.name.toLowerCase().includes(query) ||
          obj.tags.some(
            (t) =>
              t.key.toLowerCase().includes(query) ||
              t.value.toLowerCase().includes(query)
          )
        );
      })
      .filter(matchesType)
      .filter(matchesTrust)
      .filter(matchesTags);
  }, [filters.onlyTagged, filters.tag, filters.trustMin, filters.type, objects, searchQuery]);

  const sortedObjects = useMemo(() => {
    const direction = sort.direction === "asc" ? 1 : -1;
    return [...filteredObjects].sort((a, b) => {
      let comparison = 0;
      switch (sort.field) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "extension":
          comparison = (a.extension || "").localeCompare(b.extension || "");
          break;
        case "sizeBytes":
          comparison = a.sizeBytes - b.sizeBytes;
          break;
        case "modifiedAt":
          comparison = a.modifiedAt - b.modifiedAt;
          break;
        case "createdAt":
          comparison = a.createdAt - b.createdAt;
          break;
        case "trustLevel":
          comparison = (a.trustLevel ?? 0) - (b.trustLevel ?? 0);
          break;
      }
      return comparison * direction;
    });
  }, [filteredObjects, sort.direction, sort.field]);

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <Loader2 className="w-8 h-8 animate-spin" />
          <span className="text-sm">Objekte werden geladen...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-destructive">
          <span className="text-sm">Fehler beim Laden der Objekte</span>
          <span className="text-xs text-muted-foreground">{error.message}</span>
        </div>
      </div>
    );
  }

  if (sortedObjects.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <FolderOpen className="w-12 h-12 opacity-40" />
          <span className="text-sm">
            {searchQuery || filters.tag || filters.onlyTagged || filters.trustMin !== null || filters.type !== "all"
              ? "Keine Objekte passen zu deinen Filtern"
              : "Keine Objekte in dieser Ansicht"}
          </span>
        </div>
      </div>
    );
  }

  const viewProps = {
    objects: sortedObjects,
    selectedObjects,
    onObjectSelect,
    onObjectOpen,
    onObjectFocus,
    onRequestAddTag,
    onRemoveTag,
    onSetTrust,
    onShowDetails,
  };

  return (
    <div className="flex-1 overflow-hidden">
      {viewMode === "graph" && <GraphView {...viewProps} />}
      {viewMode === "grid" && <GridView {...viewProps} />}
      {viewMode === "list" && (
        <ListView {...viewProps} sort={sort} onSortChange={onSortChange} />
      )}
    </div>
  );
};
