import { useState } from "react";
import { useViewById } from "@/hooks/useViews";
import { ViewSelector } from "./ViewSelector";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Search,
  SlidersHorizontal,
  ArrowUpDown,
  Import,
  MoreHorizontal,
  ArrowDown,
  ArrowUp,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/animate-ui/components/radix/dropdown-menu";
import type { ViewMode } from "./NexusLayout";
import { ImportDialog } from "./ImportDialog";
import type { FilterState, SortField, SortState } from "./NexusLayout";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/animate-ui/components/radix/popover";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

interface ToolbarProps {
  currentViewId?: string;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  sort: SortState;
  onSortChange: (next: SortState) => void;
  filters: FilterState;
  onFiltersChange: (next: FilterState) => void;
}

export const Toolbar = ({
  currentViewId,
  viewMode,
  onViewModeChange,
  searchQuery,
  onSearchChange,
  sort,
  onSortChange,
  filters,
  onFiltersChange,
}: ToolbarProps) => {
  const { data: currentView } = useViewById(currentViewId);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const SortIcon = sort.direction === "asc" ? ArrowUp : ArrowDown;

  const handleSortFieldChange = (field: SortField) => {
    if (sort.field === field) {
      onSortChange({
        field,
        direction: sort.direction === "asc" ? "desc" : "asc",
      });
    } else {
      onSortChange({ field, direction: "asc" });
    }
  };

  return (
    <>
    
    <div className="h-12 flex-shrink-0 border-b border-border/50 flex items-center gap-3 px-4">
      {/* Current view name */}
      <div className="flex items-center gap-2 min-w-0">
        <h1 className="text-sm font-semibold truncate">
          {currentView?.name || "Alle Objekte"}
        </h1>
        {currentView && (
          <span className="text-xs text-muted-foreground">
            {currentView.objectCount} {currentView.objectCount === 1 ? "Objekt" : "Objekte"}
          </span>
        )}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Search */}
      <div className="relative w-64">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <Input
          type="text"
          placeholder="Objekte suchen..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="h-8 pl-8 text-sm bg-muted/30 border-transparent focus:border-primary/50"
        />
      </div>

      {/* View mode selector */}
      <ViewSelector value={viewMode} onChange={onViewModeChange} />

      {/* Sort button */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <ArrowUpDown className="w-4 h-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-52">
          <DropdownMenuLabel>Sortieren nach</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            value={sort.field}
            onValueChange={(value) => handleSortFieldChange(value as SortField)}
          >
            {(
              [
                { label: "Name", value: "name" },
                { label: "Typ", value: "extension" },
                { label: "Größe", value: "sizeBytes" },
                { label: "Geändert", value: "modifiedAt" },
                { label: "Erstellt", value: "createdAt" },
                { label: "Sicherheitsgrad", value: "trustLevel" },
              ] as Array<{ label: string; value: SortField }>
            ).map((option) => (
              <DropdownMenuRadioItem key={option.value} value={option.value}>
                {option.label}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onSelect={() =>
              onSortChange({
                field: sort.field,
                direction: sort.direction === "asc" ? "desc" : "asc",
              })
            }
          >
            <SortIcon className="w-4 h-4 mr-2" />
            Sortierreihenfolge umkehren
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {/* Filter button */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <SlidersHorizontal className="w-4 h-4" />
          </Button>
        </PopoverTrigger>
        <PopoverContent align="end" className="w-72">
          <div className="space-y-4">
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                Dateityp
              </Label>
              <Select
                value={filters.type}
                onValueChange={(value) =>
                  onFiltersChange({ ...filters, type: value as FilterState["type"] })
                }
              >
                <SelectTrigger className="h-8">
                  <SelectValue placeholder="Alle Typen" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Alle Typen</SelectItem>
                  <SelectItem value="document">Dokumente</SelectItem>
                  <SelectItem value="image">Bilder</SelectItem>
                  <SelectItem value="video">Videos</SelectItem>
                  <SelectItem value="audio">Audio</SelectItem>
                  <SelectItem value="code">Code</SelectItem>
                  <SelectItem value="archive">Archive</SelectItem>
                  <SelectItem value="other">Andere</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                Mindest-Sicherheitsgrad
              </Label>
              <Slider
                value={[filters.trustMin ?? 0]}
                min={0}
                max={100}
                step={5}
                onValueChange={(value) =>
                  onFiltersChange({ ...filters, trustMin: value[0] })
                }
              />
              <div className="text-xs text-muted-foreground">
                {filters.trustMin === null ? "Beliebig" : `${filters.trustMin}% oder höher`}
              </div>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 text-xs"
                onClick={() => onFiltersChange({ ...filters, trustMin: null })}
              >
                Sicherheitsfilter zurücksetzen
              </Button>
            </div>

            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                Eigenschaft enthält
              </Label>
              <Input
                placeholder="Eigenschaften suchen..."
                value={filters.tag}
                onChange={(e) =>
                  onFiltersChange({ ...filters, tag: e.target.value })
                }
                className="h-8"
              />
            </div>

            <div className="flex items-center justify-between">
              <Label htmlFor="only-tagged" className="text-sm">
                Nur Objekte mit Eigenschaften
              </Label>
              <Switch
                id="only-tagged"
                checked={filters.onlyTagged}
                onCheckedChange={(value) =>
                  onFiltersChange({ ...filters, onlyTagged: value })
                }
              />
            </div>
          </div>
        </PopoverContent>
      </Popover>

      {/* Actions menu */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <MoreHorizontal className="w-4 h-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-48">
          <DropdownMenuItem onClick={() => setImportDialogOpen(true)}>
            <Import className="w-4 h-4 mr-2" />
            Dateien importieren
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem>Perspektive exportieren</DropdownMenuItem>
          <DropdownMenuItem>Perspektive teilen</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    {/* Import Dialog */}
    <ImportDialog
      open={importDialogOpen}
      onOpenChange={setImportDialogOpen}
    />
    </>
  );
};
