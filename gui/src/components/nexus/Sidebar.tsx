import { cn } from "@/lib/utils";
import { useViews } from "@/hooks/useViews";
import {
  Clock,
  Folder,
  FileEdit,
  Eye,
  CheckCircle,
  Grid,
  Plus,
  Settings,
  ChevronDown,
  MoreVertical,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { deleteView, type ChildPolicy, type ViewInfo } from "@/lib/lfs";
import { NewViewDialog } from "./NewViewDialog";
import { EditViewDialog } from "./EditViewDialog";
import { toast } from "sonner";

const iconMap: Record<string, React.ElementType> = {
  Clock,
  Folder,
  FileEdit,
  Eye,
  CheckCircle,
  Grid,
};

interface SidebarProps {
  currentViewId?: string;
  onViewSelect: (viewId: string) => void;
  onOpenSettings: () => void;
}

interface ViewItemProps {
  view: ViewInfo;
  isActive: boolean;
  onClick: () => void;
  actions?: React.ReactNode;
  depth?: number;
}

const ViewItem = ({ view, isActive, onClick, actions, depth = 0 }: ViewItemProps) => {
  const Icon = view.icon ? iconMap[view.icon] || Folder : Folder;

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      className={cn(
        "group w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-sm",
        "transition-colors duration-150",
        "hover:bg-muted/60",
        isActive && "bg-primary/10 text-primary hover:bg-primary/15"
      )}
      style={{ paddingLeft: `${12 + depth * 14}px` }}
    >
      <Icon className="w-4 h-4 flex-shrink-0" />
      <span className="flex-1 text-left truncate">{view.name}</span>
      <span
        className={cn(
          "text-xs tabular-nums",
          isActive ? "text-primary/70" : "text-muted-foreground"
        )}
      >
        {view.objectCount}
      </span>
      {actions && (
        <div
          className={cn(
            "flex items-center overflow-hidden w-0 opacity-0 ml-0 pointer-events-none",
            "transition-[width,opacity,margin] duration-150 ease-out",
            "group-hover:w-6 group-hover:opacity-100 group-hover:ml-1 group-hover:pointer-events-auto",
            "group-focus-within:w-6 group-focus-within:opacity-100 group-focus-within:ml-1 group-focus-within:pointer-events-auto"
          )}
          onClick={(event) => event.stopPropagation()}
          onPointerDown={(event) => event.stopPropagation()}
        >
          {actions}
        </div>
      )}
    </div>
  );
};

export interface DynamicRow {
  view: ViewInfo;
  depth: number;
}

export function buildDynamicRows(dynamicViews: ViewInfo[]): DynamicRow[] {
  const byParent = new Map<string | null, ViewInfo[]>();
  for (const view of dynamicViews) {
    const key = view.parentId ?? null;
    const entries = byParent.get(key) || [];
    entries.push(view);
    byParent.set(key, entries);
  }

  for (const entries of byParent.values()) {
    entries.sort((a, b) => a.name.localeCompare(b.name));
  }

  const rows: DynamicRow[] = [];
  const visited = new Set<string>();
  const append = (parentId: string | null, depth: number) => {
    const children = byParent.get(parentId) || [];
    for (const view of children) {
      if (visited.has(view.id)) continue;
      visited.add(view.id);
      rows.push({ view, depth });
      append(view.id, depth + 1);
    }
  };
  append(null, 0);
  for (const view of dynamicViews) {
    if (visited.has(view.id)) continue;
    rows.push({ view, depth: 0 });
    append(view.id, 1);
  }
  return rows;
}

export const Sidebar = ({ currentViewId, onViewSelect, onOpenSettings }: SidebarProps) => {
  const { data: views, isLoading } = useViews();
  const queryClient = useQueryClient();
  const [builtinOpen, setBuiltinOpen] = useState(true);
  const [dynamicOpen, setDynamicOpen] = useState(true);
  const [newViewDialogOpen, setNewViewDialogOpen] = useState(false);
  const [editingView, setEditingView] = useState<ViewInfo | null>(null);
  const [pendingDelete, setPendingDelete] = useState<ViewInfo | null>(null);
  const [isDeletingView, setIsDeletingView] = useState(false);

  const builtinViews = views?.filter((v) => v.viewType === "builtin") || [];
  const dynamicViews = views?.filter((v) => v.viewType === "dynamic") || [];
  const dynamicRows = useMemo(() => buildDynamicRows(dynamicViews), [dynamicViews]);
  const pendingChildren = useMemo(
    () => (pendingDelete
      ? dynamicViews.filter((view) => view.parentId === pendingDelete.id)
      : []),
    [dynamicViews, pendingDelete]
  );
  const pendingHasChildren = pendingChildren.length > 0;

  const handleViewCreated = (viewId: string) => {
    onViewSelect(viewId);
    setDynamicOpen(true);
  };

  const handleDelete = async (policy?: ChildPolicy) => {
    if (!pendingDelete) return;
    setIsDeletingView(true);
    try {
      await deleteView(pendingDelete.id, policy);
      await queryClient.invalidateQueries({ queryKey: ["views"] });
      await queryClient.invalidateQueries({ queryKey: ["view-objects", pendingDelete.id] });
      if (currentViewId === pendingDelete.id) {
        onViewSelect("recent");
      }
      toast.success("Perspektive gelöscht");
      setPendingDelete(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Perspektive konnte nicht gelöscht werden");
    } finally {
      setIsDeletingView(false);
    }
  };

  return (
    <div className="w-60 flex-shrink-0 border-r border-border/50 flex flex-col bg-background/50">
      <ScrollArea className="flex-1 py-2">
        {/* Built-in Views */}
        <Collapsible open={builtinOpen} onOpenChange={setBuiltinOpen}>
          <CollapsibleTrigger className="flex items-center gap-1.5 w-full px-3 py-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider hover:text-foreground transition-colors">
            <ChevronDown
              className={cn(
                "w-3 h-3 transition-transform",
                !builtinOpen && "-rotate-90"
              )}
            />
            Perspektiven
          </CollapsibleTrigger>
          <CollapsibleContent className="px-2 space-y-0.5 overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
            {isLoading ? (
              <div className="px-3 py-2 text-sm text-muted-foreground">
                Lädt...
              </div>
            ) : (
              builtinViews.map((view) => (
                <ViewItem
                  key={view.id}
                  view={view}
                  isActive={currentViewId === view.id}
                  onClick={() => onViewSelect(view.id)}
                />
              ))
            )}
          </CollapsibleContent>
        </Collapsible>

        {/* Dynamic Views */}
        {dynamicViews.length > 0 && (
          <Collapsible open={dynamicOpen} onOpenChange={setDynamicOpen} className="mt-4">
            <CollapsibleTrigger className="flex items-center gap-1.5 w-full px-3 py-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider hover:text-foreground transition-colors">
              <ChevronDown
                className={cn(
                  "w-3 h-3 transition-transform",
                  !dynamicOpen && "-rotate-90"
                )}
              />
              Eigene Perspektiven
            </CollapsibleTrigger>
            <CollapsibleContent className="px-2 space-y-0.5 overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
              {dynamicRows.map(({ view, depth }) => (
                <ViewItem
                  key={view.id}
                  view={view}
                  depth={depth}
                  isActive={currentViewId === view.id}
                  onClick={() => onViewSelect(view.id)}
                  actions={(
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
                      <DropdownMenuContent align="end" className="w-40">
                        <DropdownMenuItem onSelect={() => setEditingView(view)}>
                          Bearbeiten
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          className="text-destructive focus:text-destructive"
                          onSelect={() => setPendingDelete(view)}
                        >
                          Löschen
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  )}
                />
              ))}
            </CollapsibleContent>
          </Collapsible>
        )}
      </ScrollArea>

      {/* Sidebar footer */}
      <div className="p-2 border-t border-border/50 flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          className="flex-1 justify-start gap-2 h-8 text-muted-foreground hover:text-foreground"
          onClick={() => setNewViewDialogOpen(true)}
        >
          <Plus className="w-4 h-4" />
          <span className="text-sm">Neue Perspektive</span>
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground hover:text-foreground"
          onClick={onOpenSettings}
        >
          <Settings className="w-4 h-4" />
        </Button>
      </div>

      {/* New View Dialog */}
      <NewViewDialog
        open={newViewDialogOpen}
        onOpenChange={setNewViewDialogOpen}
        onViewCreated={handleViewCreated}
      />

      <EditViewDialog
        open={!!editingView}
        view={editingView}
        onOpenChange={(open) => {
          if (!open) setEditingView(null);
        }}
      />
      <AlertDialog
        open={!!pendingDelete}
        onOpenChange={(open) => {
          if (!open && !isDeletingView) {
            setPendingDelete(null);
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Perspektive löschen</AlertDialogTitle>
            <AlertDialogDescription>
              {pendingDelete && pendingHasChildren
                ? `"${pendingDelete.name}" hat ${pendingChildren.length} Unteransicht${pendingChildren.length === 1 ? "" : "en"}. Wähle, wie mit den Unteransichten umgegangen werden soll.`
                : pendingDelete
                  ? `"${pendingDelete.name}" wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden.`
                  : "Diese Perspektive wirklich löschen?"}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeletingView}>Abbrechen</AlertDialogCancel>
            {pendingHasChildren ? (
              <>
                <Button
                  variant="outline"
                  disabled={isDeletingView}
                  onClick={() => handleDelete("detach")}
                >
                  Kinder behalten
                </Button>
                <Button
                  variant="destructive"
                  disabled={isDeletingView}
                  onClick={() => handleDelete("cascade")}
                >
                  Unteransichten löschen
                </Button>
              </>
            ) : (
              <Button
                variant="destructive"
                disabled={isDeletingView}
                onClick={() => handleDelete()}
              >
                Löschen
              </Button>
            )}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
