import { cn } from "@/lib/utils";
import type { TagInfo } from "@/lib/lfs";
import { formatMetadataKeyLabel } from "@/lib/metadataDisplay";
import { ChevronDown } from "lucide-react";
import { useMemo, useState } from "react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";

interface TagDetailsSectionProps {
  title: string;
  tags: TagInfo[];
  prefix: string;
  labelFormatter?: (raw: string) => string;
}

export const TagDetailsSection = ({
  title,
  tags,
  prefix,
  labelFormatter = formatMetadataKeyLabel,
}: TagDetailsSectionProps) => {
  const [open, setOpen] = useState(false);
  const details = useMemo(
    () =>
      tags
        .map((tag) => ({
          key: `${tag.key}:${tag.value}`,
          label: labelFormatter(tag.key.replace(prefix, "")),
          value: tag.value,
        }))
        .sort((a, b) =>
          a.label.localeCompare(b.label, undefined, { sensitivity: "base", numeric: true }),
        ),
    [tags, prefix],
  );

  if (tags.length === 0) return null;

  return (
    <section className="space-y-3">
      <Collapsible open={open} onOpenChange={setOpen}>
        <div className="flex items-center justify-between">
          <CollapsibleTrigger className="flex items-center gap-1.5 text-xs font-semibold text-foreground/75 uppercase tracking-wider hover:text-foreground transition-colors">
            <ChevronDown
              className={cn(
                "w-3 h-3 transition-transform",
                !open && "-rotate-90",
              )}
            />
            {title}
          </CollapsibleTrigger>
          <span className="text-xs text-muted-foreground">{tags.length}</span>
        </div>
        <CollapsibleContent className="pt-2 overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
          <div className="space-y-2 text-xs">
            {details.map((detail) => (
              <div
                key={detail.key}
                className="flex items-center justify-between gap-2"
              >
                <span className="text-foreground/75">{detail.label}</span>
                <span
                  className="font-medium text-right truncate max-w-[180px]"
                  title={detail.value}
                >
                  {detail.value}
                </span>
              </div>
            ))}
          </div>
        </CollapsibleContent>
      </Collapsible>
    </section>
  );
};
