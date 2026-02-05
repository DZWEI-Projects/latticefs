import { useMemo, useState, useEffect } from "react";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import {
  getFilesInView,
  mockFiles,
  mockViews,
  type FileNode,
  type ViewNode,
} from "@/data/mockFileSystem";
import {
  Boxes,
  Folder,
  LayoutGrid,
  List,
  Network,
  Search,
  Sparkles,
  Upload,
  Settings,
  ShieldCheck,
  Clock,
  Star,
  Tag,
  FileText,
  FileImage,
  FileArchive,
  FileSpreadsheet,
  FileCode,
  Presentation,
} from "lucide-react";

const viewTypeOptions = [
  { id: "graph", label: "Graph", icon: Network },
  { id: "grid", label: "Grid", icon: LayoutGrid },
  { id: "list", label: "List", icon: List },
] as const;

type ViewType = (typeof viewTypeOptions)[number]["id"];

const viewIcons: Record<string, typeof Folder> = {
  Clock,
  Folder,
  Grid: LayoutGrid,
  Download: Upload,
  Shield: ShieldCheck,
};

const fileTypeIcons: Record<string, typeof FileText> = {
  pdf: FileText,
  docx: FileText,
  md: FileText,
  jpg: FileImage,
  png: FileImage,
  xlsx: FileSpreadsheet,
  pptx: Presentation,
  zip: FileArchive,
  exe: FileArchive,
  py: FileCode,
};

const formatSize = (bytes?: number) => {
  if (!bytes) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value > 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unitIndex]}`;
};

const formatDate = (date: Date) =>
  new Intl.DateTimeFormat("de-DE", { dateStyle: "medium" }).format(date);

const GraphView = ({
  views,
  selectedView,
  onSelectView,
  files,
}: {
  views: ViewNode[];
  selectedView: ViewNode;
  onSelectView: (viewId: string) => void;
  files: FileNode[];
}) => {
  const NODE_CONTAINER_SIZE = 520;
  const NODE_CENTER = NODE_CONTAINER_SIZE / 2;

  const getViewPosition = (index: number, total: number) => {
    const angle = (index / total) * Math.PI * 2 - Math.PI / 2;
    const radius = 170;
    return {
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
    };
  };

  const selectedIndex = views.findIndex((view) => view.id === selectedView.id);
  const selectedPos = getViewPosition(Math.max(0, selectedIndex), views.length);

  return (
    <div className="relative flex items-center justify-center min-h-[480px] rounded-2xl bg-gradient-to-br from-background via-background to-background-deep border border-border/60 overflow-hidden">
      <div className="absolute inset-0 opacity-70">
        <div className="absolute left-10 top-10 h-32 w-32 rounded-full bg-primary/10 blur-3xl" />
        <div className="absolute right-10 bottom-10 h-40 w-40 rounded-full bg-secondary/10 blur-3xl" />
      </div>
      <div className="relative" style={{ width: NODE_CONTAINER_SIZE, height: NODE_CONTAINER_SIZE }}>
        <svg className="absolute inset-0 w-full h-full pointer-events-none overflow-visible">
          <defs>
            <linearGradient id="atlas-connection" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="hsl(var(--primary))" stopOpacity="0.4" />
              <stop offset="100%" stopColor="hsl(var(--secondary))" stopOpacity="0.4" />
            </linearGradient>
          </defs>
          <g transform={`translate(${NODE_CENTER}, ${NODE_CENTER})`}>
            {views.map((view, index) => {
              const pos = getViewPosition(index, views.length);
              return (
                <line
                  key={`line-${view.id}`}
                  x1="0"
                  y1="0"
                  x2={pos.x}
                  y2={pos.y}
                  stroke="url(#atlas-connection)"
                  strokeWidth="1"
                  className="opacity-50"
                />
              );
            })}
            {files.slice(0, 8).map((file, index) => {
              const angle = (index / Math.max(1, Math.min(files.length, 8))) * Math.PI * 2;
              const radius = 46 + index * 6;
              return (
                <line
                  key={`file-line-${file.id}`}
                  x1={selectedPos.x}
                  y1={selectedPos.y}
                  x2={selectedPos.x + Math.cos(angle) * radius}
                  y2={selectedPos.y + Math.sin(angle) * radius}
                  stroke="hsl(var(--primary))"
                  strokeOpacity="0.4"
                  strokeWidth="1"
                />
              );
            })}
          </g>
        </svg>
        <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
          <div className="w-16 h-16 rounded-full bg-primary/20 border border-primary/40 flex items-center justify-center">
            <div className="w-8 h-8 rounded-full bg-primary/40 flex items-center justify-center">
              <Sparkles className="w-4 h-4 text-primary" />
            </div>
          </div>
          <span className="block text-xs text-center text-muted-foreground mt-2">Lattice Atlas</span>
        </div>
        {views.map((view, index) => {
          const pos = getViewPosition(index, views.length);
          const Icon = viewIcons[view.icon] ?? Folder;
          const isActive = view.id === selectedView.id;
          return (
            <button
              key={view.id}
              type="button"
              onClick={() => onSelectView(view.id)}
              className={cn(
                "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
                "rounded-xl border px-3 py-2 text-xs transition-all",
                isActive
                  ? "border-primary/60 bg-primary/20 text-primary glow-primary"
                  : "border-border/60 bg-muted/40 text-foreground hover:border-primary/40"
              )}
              style={{
                transform: `translate(calc(-50% + ${pos.x}px), calc(-50% + ${pos.y}px))`,
              }}
            >
              <div className="flex items-center gap-2">
                <Icon className="h-4 w-4" />
                <span className="font-medium">{view.name}</span>
              </div>
              <span className="block text-[10px] text-muted-foreground mt-1">
                {view.files.length} Objekte
              </span>
            </button>
          );
        })}
        {files.slice(0, 8).map((file, index) => {
          const angle = (index / Math.max(1, Math.min(files.length, 8))) * Math.PI * 2 - Math.PI / 2;
          const radius = 48 + index * 7;
          const FileIcon = fileTypeIcons[file.extension ?? ""] ?? FileText;
          return (
            <div
              key={`file-${file.id}`}
              className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 text-[10px]"
              style={{
                transform: `translate(calc(-50% + ${selectedPos.x + Math.cos(angle) * radius}px), calc(-50% + ${selectedPos.y + Math.sin(angle) * radius}px))`,
              }}
            >
              <div className="flex items-center gap-1 rounded-md border border-border/60 bg-muted/60 px-2 py-1">
                <FileIcon className="h-3 w-3 text-muted-foreground" />
                <span className="max-w-[72px] truncate text-foreground/80">
                  {file.name}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

const FileListView = ({
  files,
  selectedFile,
  onSelectFile,
}: {
  files: FileNode[];
  selectedFile?: FileNode;
  onSelectFile: (file: FileNode) => void;
}) => (
  <div className="rounded-2xl border border-border/60 overflow-hidden bg-card/60">
    <div className="grid grid-cols-[2.4fr_1fr_1fr_0.8fr] gap-4 px-4 py-3 text-[11px] uppercase tracking-wide text-muted-foreground border-b border-border/60">
      <span>Name</span>
      <span>Tags</span>
      <span>Geändert</span>
      <span>Größe</span>
    </div>
    <div className="divide-y divide-border/60">
      {files.map((file) => {
        const FileIcon = fileTypeIcons[file.extension ?? ""] ?? FileText;
        const isSelected = selectedFile?.id === file.id;
        return (
          <button
            key={file.id}
            type="button"
            onClick={() => onSelectFile(file)}
            className={cn(
              "w-full grid grid-cols-[2.4fr_1fr_1fr_0.8fr] gap-4 px-4 py-3 text-left",
              "transition-colors",
              isSelected ? "bg-primary/10" : "hover:bg-muted/40"
            )}
          >
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-lg border border-border/60 bg-muted/70 flex items-center justify-center">
                <FileIcon className="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <div className="font-medium text-foreground truncate max-w-[260px]">
                  {file.name}
                </div>
                <div className="text-xs text-muted-foreground">{file.extension?.toUpperCase()}</div>
              </div>
            </div>
            <div className="flex flex-wrap gap-1">
              {file.tags.slice(0, 2).map((tag) => (
                <Badge key={tag} variant="secondary" className="text-[10px]">
                  {tag}
                </Badge>
              ))}
            </div>
            <div className="text-sm text-muted-foreground">{formatDate(file.modifiedAt)}</div>
            <div className="text-sm text-muted-foreground">{formatSize(file.size)}</div>
          </button>
        );
      })}
    </div>
  </div>
);

const FileGridView = ({
  files,
  selectedFile,
  onSelectFile,
}: {
  files: FileNode[];
  selectedFile?: FileNode;
  onSelectFile: (file: FileNode) => void;
}) => (
  <div className="grid grid-cols-3 gap-4">
    {files.map((file) => {
      const FileIcon = fileTypeIcons[file.extension ?? ""] ?? FileText;
      const isSelected = selectedFile?.id === file.id;
      return (
        <button
          key={file.id}
          type="button"
          onClick={() => onSelectFile(file)}
          className={cn(
            "rounded-2xl border border-border/60 bg-card/60 p-4 text-left transition-all",
            isSelected ? "border-primary/60 bg-primary/10 glow-primary" : "hover:border-primary/40"
          )}
        >
          <div className="flex items-start justify-between gap-3">
            <div className="h-10 w-10 rounded-xl border border-border/60 bg-muted/70 flex items-center justify-center">
              <FileIcon className="h-5 w-5 text-muted-foreground" />
            </div>
            <span className="text-xs text-muted-foreground">{formatSize(file.size)}</span>
          </div>
          <div className="mt-3">
            <div className="font-medium text-foreground truncate">{file.name}</div>
            <div className="text-xs text-muted-foreground mt-1">{formatDate(file.modifiedAt)}</div>
          </div>
          <div className="mt-3 flex flex-wrap gap-1">
            {file.tags.slice(0, 3).map((tag) => (
              <Badge key={tag} variant="secondary" className="text-[10px]">
                {tag}
              </Badge>
            ))}
          </div>
        </button>
      );
    })}
  </div>
);

const Atlas = () => {
  const [viewType, setViewType] = useState<ViewType>("graph");
  const [selectedViewId, setSelectedViewId] = useState(mockViews[0]?.id ?? "");
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null);

  const selectedView = useMemo(
    () => mockViews.find((view) => view.id === selectedViewId) ?? mockViews[0],
    [selectedViewId]
  );
  const filesInView = useMemo(() => getFilesInView(selectedView?.id ?? ""), [selectedView]);
  const selectedFile = useMemo(
    () => filesInView.find((file) => file.id === selectedFileId) ?? filesInView[0],
    [filesInView, selectedFileId]
  );

  useEffect(() => {
    if (!filesInView.length) return;
    if (selectedFileId && filesInView.some((file) => file.id === selectedFileId)) {
      return;
    }
    setSelectedFileId(filesInView[0].id);
  }, [filesInView, selectedFileId]);

  return (
    <div className="flex h-screen w-full overflow-hidden text-foreground">
      <aside className="w-72 border-r border-border/60 bg-sidebar/80 backdrop-blur-xl p-5 flex flex-col gap-6">
        <div className="flex items-center gap-3">
          <div className="h-10 w-10 rounded-xl bg-primary/20 border border-primary/40 flex items-center justify-center">
            <Boxes className="h-5 w-5 text-primary" />
          </div>
          <div>
            <div className="text-sm uppercase tracking-[0.2em] text-muted-foreground">Atlas</div>
            <div className="text-lg font-semibold">Lattice Hub</div>
          </div>
        </div>

        <div className="space-y-3">
          <div className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Ansichten</div>
          <div className="flex flex-col gap-2">
            {mockViews.map((view) => {
              const Icon = viewIcons[view.icon] ?? Folder;
              const isActive = view.id === selectedView.id;
              return (
                <button
                  key={view.id}
                  type="button"
                  onClick={() => setSelectedViewId(view.id)}
                  className={cn(
                    "flex items-center justify-between gap-3 rounded-xl border border-transparent px-3 py-2 transition-colors",
                    isActive
                      ? "bg-primary/15 border-primary/40 text-primary"
                      : "hover:bg-muted/50"
                  )}
                >
                  <div className="flex items-center gap-2">
                    <Icon className="h-4 w-4" />
                    <span className="text-sm font-medium text-foreground">{view.name}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">{view.files.length}</span>
                </button>
              );
            })}
          </div>
        </div>

        <div className="space-y-3">
          <div className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Schnellzugriff</div>
          <div className="flex flex-col gap-2">
            <button type="button" className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground">
              <Clock className="h-4 w-4" />
              Zuletzt geöffnet
            </button>
            <button type="button" className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground">
              <Star className="h-4 w-4" />
              Favoriten
            </button>
            <button type="button" className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground">
              <Tag className="h-4 w-4" />
              Tags & Filter
            </button>
          </div>
        </div>

        <div className="mt-auto space-y-3">
          <div className="rounded-xl border border-border/60 bg-muted/50 p-3">
            <div className="text-xs text-muted-foreground">Sync-Status</div>
            <div className="mt-1 flex items-center gap-2 text-sm">
              <ShieldCheck className="h-4 w-4 text-primary" />
              Gesichert · Letzte Prüfung vor 2 Min
            </div>
          </div>
          <Button variant="outline" className="w-full justify-start gap-2">
            <Settings className="h-4 w-4" />
            Einstellungen
          </Button>
        </div>
      </aside>

      <div className="flex-1 flex flex-col">
        <header className="border-b border-border/60 bg-card/40 px-6 py-4">
          <div className="flex items-center justify-between gap-6">
            <div className="flex flex-col gap-2">
              <Breadcrumb>
                <BreadcrumbList>
                  <BreadcrumbItem>
                    <BreadcrumbPage>Lattice</BreadcrumbPage>
                  </BreadcrumbItem>
                  <BreadcrumbSeparator />
                  <BreadcrumbItem>
                    <BreadcrumbPage>Atlas</BreadcrumbPage>
                  </BreadcrumbItem>
                  <BreadcrumbSeparator />
                  <BreadcrumbItem>
                    <BreadcrumbPage>{selectedView?.name}</BreadcrumbPage>
                  </BreadcrumbItem>
                </BreadcrumbList>
              </Breadcrumb>
              <div className="text-sm text-muted-foreground">
                {selectedView?.description}
              </div>
            </div>

            <div className="flex items-center gap-3">
              <div className="relative">
                <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Suchen in Atlas…"
                  className="pl-9 w-[240px]"
                />
              </div>
              <Button variant="outline" className="gap-2">
                <Upload className="h-4 w-4" />
                Importieren
              </Button>
              <Button className="gap-2">
                <Sparkles className="h-4 w-4" />
                Neue Ansicht
              </Button>
            </div>
          </div>

          <div className="mt-4 flex items-center justify-between">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <span className="rounded-full bg-muted/60 px-2 py-1">{filesInView.length} Objekte</span>
              <span className="rounded-full bg-muted/60 px-2 py-1">{mockFiles.length} insgesamt</span>
              <span className="rounded-full bg-muted/60 px-2 py-1">{mockViews.length} Ansichten</span>
            </div>
            <Tooltip>
              <TooltipTrigger asChild>
                <div className="flex items-center gap-2">
                  <span className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Ansicht</span>
                  <ToggleGroup
                    type="single"
                    value={viewType}
                    onValueChange={(value) => value && setViewType(value as ViewType)}
                    className="rounded-full border border-border/60 bg-muted/40 px-1"
                  >
                    {viewTypeOptions.map((option) => (
                      <ToggleGroupItem
                        key={option.id}
                        value={option.id}
                        className="rounded-full px-3"
                      >
                        <option.icon className="h-4 w-4" />
                      </ToggleGroupItem>
                    ))}
                  </ToggleGroup>
                </div>
              </TooltipTrigger>
              <TooltipContent side="bottom" className="max-w-[220px] text-xs">
                Standardansicht ist der Graph. Falls dir das zu ungewohnt ist, kannst du hier jederzeit auf Grid oder Liste wechseln.
              </TooltipContent>
            </Tooltip>
          </div>
        </header>

        <div className="flex-1 flex overflow-hidden">
          <ScrollArea className="flex-1 px-6 py-6">
            {viewType === "graph" && selectedView && (
              <GraphView
                views={mockViews}
                selectedView={selectedView}
                onSelectView={setSelectedViewId}
                files={filesInView}
              />
            )}
            {viewType === "list" && (
              <FileListView
                files={filesInView}
                selectedFile={selectedFile}
                onSelectFile={(file) => setSelectedFileId(file.id)}
              />
            )}
            {viewType === "grid" && (
              <FileGridView
                files={filesInView}
                selectedFile={selectedFile}
                onSelectFile={(file) => setSelectedFileId(file.id)}
              />
            )}
          </ScrollArea>

          <aside className="w-80 border-l border-border/60 bg-card/40 p-5 hidden xl:flex flex-col gap-4">
            <div className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Inspector</div>
            {selectedFile ? (
              <div className="rounded-2xl border border-border/60 bg-muted/40 p-4 space-y-3">
                <div className="flex items-center gap-3">
                  <div className="h-12 w-12 rounded-xl border border-border/60 bg-muted/70 flex items-center justify-center">
                    {(() => {
                      const Icon = fileTypeIcons[selectedFile.extension ?? ""] ?? FileText;
                      return <Icon className="h-5 w-5 text-muted-foreground" />;
                    })()}
                  </div>
                  <div>
                    <div className="font-semibold text-foreground">{selectedFile.name}</div>
                    <div className="text-xs text-muted-foreground">
                      {selectedFile.extension?.toUpperCase()} · {formatSize(selectedFile.size)}
                    </div>
                  </div>
                </div>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center justify-between text-muted-foreground">
                    <span>Erstellt</span>
                    <span>{formatDate(selectedFile.createdAt)}</span>
                  </div>
                  <div className="flex items-center justify-between text-muted-foreground">
                    <span>Geändert</span>
                    <span>{formatDate(selectedFile.modifiedAt)}</span>
                  </div>
                  <div className="flex items-center justify-between text-muted-foreground">
                    <span>Zugriff</span>
                    <span>{formatDate(selectedFile.accessedAt)}</span>
                  </div>
                </div>
                <div>
                  <div className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Tags</div>
                  <div className="mt-2 flex flex-wrap gap-1">
                    {selectedFile.tags.map((tag) => (
                      <Badge key={tag} variant="secondary" className="text-[10px]">
                        {tag}
                      </Badge>
                    ))}
                  </div>
                </div>
                {selectedFile.metadata && (
                  <div className="text-xs text-muted-foreground space-y-1">
                    {selectedFile.metadata.project && (
                      <div>Projekt: {selectedFile.metadata.project}</div>
                    )}
                    {selectedFile.metadata.source && (
                      <div>Quelle: {selectedFile.metadata.source}</div>
                    )}
                    {selectedFile.metadata.downloadedFrom && (
                      <div>Download: {selectedFile.metadata.downloadedFrom}</div>
                    )}
                  </div>
                )}
                <div className="pt-2 flex flex-col gap-2">
                  <Button size="sm" className="w-full">Öffnen</Button>
                  <Button size="sm" variant="outline" className="w-full">Teilen</Button>
                </div>
              </div>
            ) : (
              <div className="rounded-2xl border border-border/60 bg-muted/40 p-4 text-sm text-muted-foreground">
                Wähle ein Objekt aus, um Details zu sehen.
              </div>
            )}

            <div className="rounded-2xl border border-border/60 bg-muted/40 p-4">
              <div className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Konnektivität</div>
              <div className="mt-2 text-sm text-muted-foreground">
                {selectedFile?.connections.length ?? 0} verwandte Objekte verknüpft.
              </div>
              <div className="mt-3 flex flex-wrap gap-2">
                {(selectedFile?.connections ?? []).slice(0, 3).map((connectionId) => {
                  const connectedFile = mockFiles.find((file) => file.id === connectionId);
                  if (!connectedFile) return null;
                  return (
                    <div
                      key={connectionId}
                      className="rounded-lg border border-border/60 bg-card/60 px-2 py-1 text-xs text-muted-foreground"
                    >
                      {connectedFile.name}
                    </div>
                  );
                })}
              </div>
            </div>
          </aside>
        </div>

        <footer className="border-t border-border/60 bg-card/40 px-6 py-3 text-xs text-muted-foreground flex items-center justify-between">
          <span>Bereit · LatticeFS läuft lokal</span>
          <span>Letzte Synchronisierung: vor 1 Minute</span>
        </footer>
      </div>
    </div>
  );
};

export default Atlas;
