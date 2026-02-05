import { useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cn } from "@/lib/utils";
import { Sidebar } from "./Sidebar";
import { Toolbar } from "./Toolbar";
import { ContentArea } from "./ContentArea";
import { StatusBar } from "./StatusBar";
import type { ViewInfo, ObjectInfo } from "@/lib/lfs";

export type ViewMode = "graph" | "grid" | "list";

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
  }, []);

  const handleObjectOpen = useCallback((object: ObjectInfo) => {
    // TODO: Implement object preview/open
    console.log("Open object:", object);
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
          currentViewId={currentViewId}
          onViewSelect={onViewChange}
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
          />

          {/* Content */}
          <ContentArea
            viewId={currentViewId}
            viewMode={viewMode}
            selectedObjects={selectedObjects}
            onObjectSelect={handleObjectSelect}
            onObjectOpen={handleObjectOpen}
            searchQuery={searchQuery}
          />

          {/* Status bar */}
          <StatusBar
            viewId={currentViewId}
            selectedCount={selectedObjects.length}
          />
        </div>
      </div>
    </div>
  );
};
