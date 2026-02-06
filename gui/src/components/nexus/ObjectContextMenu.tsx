import type { ReactNode } from "react";
import type { ObjectInfo, TagInfo, ViewInfo } from "@/lib/lfs";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { isTextEditable } from "@/lib/lfs";
import {
  BadgePlus,
  FileEdit,
  FolderOpen,
  History,
  Pencil,
  Shield,
  Tag,
  Trash2,
} from "lucide-react";

interface ObjectContextMenuProps {
  object: ObjectInfo;
  views?: ViewInfo[];
  onViewSelect?: (viewId: string) => void;
  onOpen: (object: ObjectInfo) => void;
  onShowDetails: (object: ObjectInfo) => void;
  onOpenVersions?: (object: ObjectInfo) => void;
  onOpenEditor?: (object: ObjectInfo) => void;
  onRename?: (object: ObjectInfo) => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  children: ReactNode;
}

const trustOptions = [
  { label: "Bestätigt", value: 100 },
  { label: "Hoch", value: 85 },
  { label: "Mittel", value: 65 },
  { label: "Niedrig", value: 40 },
  { label: "Kritisch", value: 15 },
  { label: "Nicht gesetzt", value: null },
];

export const ObjectContextMenu = ({
  object,
  views,
  onViewSelect,
  onOpen,
  onShowDetails,
  onOpenVersions,
  onOpenEditor,
  onRename,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  children,
}: ObjectContextMenuProps) => {
  const viewOptions = (views ?? [])
    .filter((view) => object.views.includes(view.id))
    .sort((a, b) => a.name.localeCompare(b.name, "de", { sensitivity: "base" }));
  const canEdit = isTextEditable(object.extension) && !object.isSealed;
  const canOpenViews = viewOptions.length > 0 && !!onViewSelect;

  return (
    <ContextMenu>
      <ContextMenuTrigger>
        <div className="contents">{children}</div>
      </ContextMenuTrigger>
      <ContextMenuContent className="w-56">
        <ContextMenuLabel className="truncate">{object.name}</ContextMenuLabel>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => onOpen(object)}>
          <FolderOpen className="w-4 h-4 mr-2" />
          Öffnen
        </ContextMenuItem>
        {canEdit && onOpenEditor && (
          <ContextMenuItem onSelect={() => onOpenEditor(object)}>
            <FileEdit className="w-4 h-4 mr-2" />
            Im Editor öffnen
          </ContextMenuItem>
        )}
        <ContextMenuItem onSelect={() => onShowDetails(object)}>
          <Shield className="w-4 h-4 mr-2" />
          Details anzeigen
        </ContextMenuItem>
        {onRename && (
          <ContextMenuItem onSelect={() => onRename(object)}>
            <Pencil className="w-4 h-4 mr-2" />
            Umbenennen
          </ContextMenuItem>
        )}
        {onOpenVersions && (
          <ContextMenuItem onSelect={() => onOpenVersions(object)}>
            <History className="w-4 h-4 mr-2" />
            Versionen prüfen
          </ContextMenuItem>
        )}
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <FolderOpen className="w-4 h-4 mr-2" />
            Perspektiven
          </ContextMenuSubTrigger>
          <ContextMenuSubContent className="w-56">
            {!canOpenViews ? (
              <ContextMenuItem disabled>Keine Perspektiven verfügbar</ContextMenuItem>
            ) : (
              viewOptions.map((view) => (
                <ContextMenuItem key={view.id} onSelect={() => onViewSelect?.(view.id)}>
                  {view.name}
                </ContextMenuItem>
              ))
            )}
          </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => onRequestAddTag(object)}>
          <BadgePlus className="w-4 h-4 mr-2" />
          Eigenschaft hinzufügen
        </ContextMenuItem>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Tag className="w-4 h-4 mr-2" />
            Eigenschaft entfernen
          </ContextMenuSubTrigger>
          <ContextMenuSubContent className="w-48">
            {object.tags.length === 0 ? (
              <ContextMenuItem disabled>Keine Eigenschaften zugewiesen</ContextMenuItem>
            ) : (
              object.tags.map((tag) => (
                <ContextMenuItem
                  key={`${tag.key}:${tag.value}`}
                  onSelect={() => onRemoveTag(object, tag)}
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  <span className="truncate">
                    {tag.key}:{tag.value}
                  </span>
                </ContextMenuItem>
              ))
            )}
          </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Shield className="w-4 h-4 mr-2" />
            Sicherheitsgrad setzen
          </ContextMenuSubTrigger>
          <ContextMenuSubContent className="w-44">
            {trustOptions.map((option) => (
              <ContextMenuItem
                key={option.label}
                onSelect={() => onSetTrust(object, option.value)}
              >
                {option.label}
              </ContextMenuItem>
            ))}
          </ContextMenuSubContent>
        </ContextMenuSub>
      </ContextMenuContent>
    </ContextMenu>
  );
};
