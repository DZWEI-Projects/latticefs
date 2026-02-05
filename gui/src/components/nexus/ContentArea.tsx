import { cn } from "@/lib/utils";
import { useViewObjects } from "@/hooks/useViewObjects";
import { GraphView } from "./GraphView";
import { GridView } from "./GridView";
import { ListView } from "./ListView";
import { Loader2, FolderOpen } from "lucide-react";
import type { ViewMode } from "./NexusLayout";
import type { ObjectInfo } from "@/lib/lfs";

interface ContentAreaProps {
  viewId?: string;
  viewMode: ViewMode;
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
  searchQuery?: string;
}

export const ContentArea = ({
  viewId,
  viewMode,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
  searchQuery,
}: ContentAreaProps) => {
  const { data: objects, isLoading, error } = useViewObjects(viewId || "all-objects");

  // Filter objects by search query
  const filteredObjects = objects?.filter((obj) => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return (
      obj.name.toLowerCase().includes(query) ||
      obj.tags.some(
        (t) =>
          t.key.toLowerCase().includes(query) ||
          t.value.toLowerCase().includes(query)
      )
    );
  });

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <Loader2 className="w-8 h-8 animate-spin" />
          <span className="text-sm">Loading objects...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-destructive">
          <span className="text-sm">Failed to load objects</span>
          <span className="text-xs text-muted-foreground">{error.message}</span>
        </div>
      </div>
    );
  }

  if (!filteredObjects || filteredObjects.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <FolderOpen className="w-12 h-12 opacity-40" />
          <span className="text-sm">
            {searchQuery ? "No objects match your search" : "No objects in this view"}
          </span>
        </div>
      </div>
    );
  }

  const viewProps = {
    objects: filteredObjects,
    selectedObjects,
    onObjectSelect,
    onObjectOpen,
  };

  return (
    <div className="flex-1 overflow-hidden">
      {viewMode === "graph" && <GraphView {...viewProps} />}
      {viewMode === "grid" && <GridView {...viewProps} />}
      {viewMode === "list" && <ListView {...viewProps} />}
    </div>
  );
};
