import { useState } from "react";
import { cn } from "@/lib/utils";
import {
  getHeatColor,
  getConnectionThickness,
  shouldApplyGlow,
} from "@/lib/graphVisualization";

interface LegendItem {
  viewCount: number;
  label: string;
}

const legendItems: LegendItem[] = [
  { viewCount: 1, label: "1" },
  { viewCount: 3, label: "3" },
  { viewCount: 6, label: "6+" },
];

export const GraphLegend = () => {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div
      className={cn(
        "absolute -bottom-6 -right-32 z-20",
        "rounded-lg border bg-card/80 backdrop-blur-sm shadow-sm",
        "p-3 transition-opacity duration-300",
        isHovered ? "opacity-100" : "opacity-20"
      )}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div className="space-y-2">
        <div className="text-xs font-medium text-foreground/75 uppercase tracking-tight">
          Verbindungsstärke
        </div>
        <span className="text-xs text-muted-foreground font-medium">
        Gemeinsame Perspektiven
        </span>
        <div className="space-y-1.5">
          {legendItems.map((item) => {
            const color = getHeatColor(item.viewCount);
            const thickness = getConnectionThickness(item.viewCount);
            const hasGlow = shouldApplyGlow(item.viewCount);
            
            return (
              <div key={item.viewCount} className="flex items-center gap-2 text-xs">
                <svg width="40" height="2" className="flex-shrink-0">
                  <line
                    x1="0"
                    y1="1"
                    x2="40"
                    y2="1"
                    stroke={color}
                    strokeWidth={thickness}
                    strokeOpacity={0.8}
                    strokeDasharray={hasGlow ? undefined : "4 2"}
                    className={hasGlow ? "drop-shadow-sm" : ""}
                    style={
                      hasGlow
                        ? {
                            filter: `drop-shadow(0 0 ${Math.min(item.viewCount * 0.5, 3)}px ${color})`,
                          }
                        : undefined
                    }
                  />
                </svg>
                <span className="text-muted-foreground whitespace-nowrap">{item.label}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
