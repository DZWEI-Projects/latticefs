import { cn } from "@/lib/utils";
import type { ViewInfo } from "@/lib/lfs";
import {
  FileItem,
  Files,
  FolderContent,
  FolderItem,
  FolderTrigger,
  SubFiles,
} from "@/components/animate-ui/components/radix/files";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { MoreVertical } from "lucide-react";
import { useMemo } from "react";

interface ViewTreeProps {
  views: ViewInfo[];
  currentViewId?: string;
  onViewSelect: (viewId: string) => void;
  onEditView: (view: ViewInfo) => void;
  onDeleteView: (view: ViewInfo) => void;
  onCreateSubView: (parentId: string) => void;
}

interface ViewTreeNodeProps {
  view: ViewInfo;
  childrenMap: Map<string, ViewInfo[]>;
  currentViewId?: string;
  onViewSelect: (viewId: string) => void;
  onEditView: (view: ViewInfo) => void;
  onDeleteView: (view: ViewInfo) => void;
  onCreateSubView: (parentId: string) => void;
}

const ViewActions = ({
  view,
  onEditView,
  onDeleteView,
  onCreateSubView,
}: {
  view: ViewInfo;
  onEditView: (view: ViewInfo) => void;
  onDeleteView: (view: ViewInfo) => void;
  onCreateSubView: (parentId: string) => void;
}) => {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 text-muted-foreground hover:text-foreground"
          aria-label={`Optionen für ${view.name}`}
        >
          <MoreVertical className="w-3.5 h-3.5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        <DropdownMenuItem onSelect={() => onCreateSubView(view.id)}>
          Neue Teilperspektive
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={() => onEditView(view)}>
          Bearbeiten
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="text-destructive focus:text-destructive"
          onSelect={() => onDeleteView(view)}
        >
          Löschen
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const ViewTreeNode = ({
  view,
  childrenMap,
  currentViewId,
  onViewSelect,
  onEditView,
  onDeleteView,
  onCreateSubView,
}: ViewTreeNodeProps) => {
  const children = childrenMap.get(view.id) ?? [];
  const hasChildren = children.length > 0;
  const isActive = currentViewId === view.id;
  const defaultOpenChildren = children
    .filter((child) => (childrenMap.get(child.id)?.length ?? 0) > 0)
    .map((child) => child.id);

  const meta = (
    <div className="absolute right-2 top-1/2 -translate-y-1/2 z-10 flex items-center gap-1">
      <span
        className={cn(
          "text-xs tabular-nums pointer-events-none",
          isActive ? "text-primary/70" : "text-muted-foreground"
        )}
      >
        {view.objectCount}
      </span>
      <div
        onClick={(event) => event.stopPropagation()}
        onPointerDown={(event) => event.stopPropagation()}
      >
        <ViewActions
          view={view}
          onEditView={onEditView}
          onDeleteView={onDeleteView}
          onCreateSubView={onCreateSubView}
        />
      </div>
    </div>
  );

  if (!hasChildren) {
    return (
      <div className="group relative">
        <button
          type="button"
          className="w-full text-left rounded-md"
          onClick={() => onViewSelect(view.id)}
        >
          <FileItem className={cn("pr-16", isActive && "text-primary font-medium")}>
            {view.name}
          </FileItem>
        </button>
        {meta}
      </div>
    );
  }

  return (
    <FolderItem value={view.id}>
      <div className="group relative" onClick={() => onViewSelect(view.id)}>
        <FolderTrigger
          className={cn(
            "w-full flex items-center justify-between pr-16",
            isActive && "text-primary font-medium"
          )}
        >
          {view.name}
        </FolderTrigger>
        {meta}
      </div>
      <FolderContent>
        <SubFiles defaultOpen={defaultOpenChildren}>
          {children.map((child) => (
            <ViewTreeNode
              key={child.id}
              view={child}
              childrenMap={childrenMap}
              currentViewId={currentViewId}
              onViewSelect={onViewSelect}
              onEditView={onEditView}
              onDeleteView={onDeleteView}
              onCreateSubView={onCreateSubView}
            />
          ))}
        </SubFiles>
      </FolderContent>
    </FolderItem>
  );
};

export const ViewTree = ({
  views,
  currentViewId,
  onViewSelect,
  onEditView,
  onDeleteView,
  onCreateSubView,
}: ViewTreeProps) => {
  const { rootViews, childrenMap, defaultOpen } = useMemo(() => {
    const viewMap = new Map(views.map((view) => [view.id, view]));
    const childMap = new Map<string, ViewInfo[]>();
    const roots: ViewInfo[] = [];

    for (const view of views) {
      if (!view.parentId || !viewMap.has(view.parentId)) {
        roots.push(view);
      } else {
        const children = childMap.get(view.parentId) ?? [];
        children.push(view);
        childMap.set(view.parentId, children);
      }
    }

    const sortByName = (items: ViewInfo[]) => {
      items.sort((a, b) => a.name.localeCompare(b.name, "de", { sensitivity: "base" }));
      for (const item of items) {
        const children = childMap.get(item.id);
        if (children) sortByName(children);
      }
    };

    sortByName(roots);

    return {
      rootViews: roots,
      childrenMap: childMap,
      defaultOpen: roots
        .filter((view) => (childMap.get(view.id)?.length ?? 0) > 0)
        .map((view) => view.id),
    };
  }, [views]);

  if (rootViews.length === 0) {
    return (
      <div className="px-3 py-2 text-sm text-muted-foreground">
        Keine eigenen Perspektiven vorhanden.
      </div>
    );
  }

  return (
    <Files className="w-full p-0" defaultOpen={defaultOpen}>
      {rootViews.map((view) => (
        <ViewTreeNode
          key={view.id}
          view={view}
          childrenMap={childrenMap}
          currentViewId={currentViewId}
          onViewSelect={onViewSelect}
          onEditView={onEditView}
          onDeleteView={onDeleteView}
          onCreateSubView={onCreateSubView}
        />
      ))}
    </Files>
  );
};
