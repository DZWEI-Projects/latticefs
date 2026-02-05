import type { ReactNode } from "react";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
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
import { BadgePlus, FolderOpen, Shield, Tag, Trash2 } from "lucide-react";

interface ObjectContextMenuProps {
  object: ObjectInfo;
  onOpen: (object: ObjectInfo) => void;
  onShowDetails: (object: ObjectInfo) => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  children: ReactNode;
}

const trustOptions = [
  { label: "Verified", value: 100 },
  { label: "High", value: 85 },
  { label: "Medium", value: 65 },
  { label: "Low", value: 40 },
  { label: "Untrusted", value: 15 },
  { label: "Unset", value: null },
];

export const ObjectContextMenu = ({
  object,
  onOpen,
  onShowDetails,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  children,
}: ObjectContextMenuProps) => {
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
          Open
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => onShowDetails(object)}>
          <Shield className="w-4 h-4 mr-2" />
          Show details
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => onRequestAddTag(object)}>
          <BadgePlus className="w-4 h-4 mr-2" />
          Add tag
        </ContextMenuItem>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Tag className="w-4 h-4 mr-2" />
            Remove tag
          </ContextMenuSubTrigger>
          <ContextMenuSubContent className="w-48">
            {object.tags.length === 0 ? (
              <ContextMenuItem disabled>No tags assigned</ContextMenuItem>
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
            Set trust
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
