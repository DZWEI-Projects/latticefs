import * as React from "react";
import * as SliderPrimitive from "@radix-ui/react-slider";

import { cn } from "@/lib/utils";

const Slider = React.forwardRef<
  React.ElementRef<typeof SliderPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof SliderPrimitive.Root>
>(({ className, ...props }, ref) => (
  <SliderPrimitive.Root
    ref={ref}
    className={cn(
      "group relative flex w-full touch-none select-none items-center py-3",
      "data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      "data-[orientation=vertical]:h-full data-[orientation=vertical]:w-3 data-[orientation=vertical]:flex-col data-[orientation=vertical]:py-0",
      className,
    )}
    {...props}
  >
    <SliderPrimitive.Track
      className={cn(
        "relative h-2 w-full grow overflow-hidden rounded-full border border-border/60 bg-muted/40",
        "shadow-[inset_0_1px_3px_hsl(var(--background)/0.6)]",
        "before:pointer-events-none before:absolute before:inset-0 before:z-0 before:bg-[linear-gradient(180deg,hsl(var(--primary)/0.18),transparent)] before:opacity-70",
        "after:pointer-events-none after:absolute after:inset-[1px] after:z-0 after:rounded-full after:bg-[linear-gradient(90deg,hsl(var(--primary)/0.08),transparent)]",
        "group-hover:border-primary/40 group-hover:bg-muted/30",
        "data-[orientation=vertical]:h-full data-[orientation=vertical]:w-2",
      )}
    >
      <SliderPrimitive.Range
        className={cn(
          "absolute h-full rounded-full bg-gradient-to-r from-primary/75 via-primary to-secondary/70",
          "shadow-[0_0_14px_hsl(var(--primary)/0.45)]",
          "after:pointer-events-none after:absolute after:inset-0 after:rounded-full after:content-['']",
          "after:bg-[linear-gradient(110deg,transparent,rgba(255,255,255,0.35),transparent)]",
          "after:opacity-40 after:animate-[shimmer_3.5s_linear_infinite]",
          "data-[orientation=vertical]:bg-gradient-to-b",
          "z-10",
        )}
      />
    </SliderPrimitive.Track>
    <SliderPrimitive.Thumb
      className={cn(
        "relative block h-6 w-6 rounded-full border border-primary/40 bg-background/35 backdrop-blur-lg backdrop-saturate-150",
        "shadow-[0_0_0_1px_hsl(var(--primary)/0.25),0_6px_18px_hsl(var(--background)/0.6)]",
        "transition-[transform,box-shadow,background-color] duration-300",
        "before:pointer-events-none before:absolute before:inset-[-3px] before:rounded-full before:content-['']",
        "before:bg-white/10 before:backdrop-blur-[12px] before:backdrop-saturate-[200%] before:opacity-70",
        "after:pointer-events-none after:absolute after:inset-[2px] after:rounded-full after:content-['']",
        "after:bg-[radial-gradient(circle_at_30%_30%,rgba(255,255,255,0.75),transparent_60%)] after:opacity-90",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
        "data-[state=active]:scale-[1.08] data-[state=active]:bg-background/30",
        "data-[state=active]:shadow-[0_0_0_1px_hsl(var(--primary)/0.6),0_0_26px_hsl(var(--primary)/0.85),0_12px_30px_hsl(var(--background)/0.6)]",
        "group-hover:shadow-[0_0_0_1px_hsl(var(--primary)/0.35),0_8px_22px_hsl(var(--primary)/0.2)]",
      )}
    />
  </SliderPrimitive.Root>
));
Slider.displayName = SliderPrimitive.Root.displayName;

export { Slider };
