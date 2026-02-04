import { cn } from "@/lib/utils";
import { useState, useEffect } from "react";

interface ToggleSwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
}

export const ToggleSwitch = ({
  checked,
  onChange,
  disabled = false,
  className,
}: ToggleSwitchProps) => {
  const [isAnimating, setIsAnimating] = useState(false);

  useEffect(() => {
    if (isAnimating) {
      const timer = setTimeout(() => setIsAnimating(false), 300);
      return () => clearTimeout(timer);
    }
  }, [isAnimating]);

  const handleClick = () => {
    if (disabled) return;
    setIsAnimating(true);
    onChange(!checked);
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={handleClick}
      className={cn(
        "relative inline-flex h-7 w-12 shrink-0 cursor-pointer items-center rounded-full",
        "transition-colors duration-300 ease-out-expo",
        "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-background",
        "disabled:cursor-not-allowed disabled:opacity-50",
        checked 
          ? "bg-primary/80" 
          : "bg-muted",
        className
      )}
    >
      {/* Track glow when active */}
      {checked && (
        <div 
          className="absolute inset-0 rounded-full opacity-40"
          style={{
            boxShadow: "0 0 12px hsl(var(--primary) / 0.6)",
          }}
        />
      )}
      
      {/* Thumb */}
      <span
        className={cn(
          "pointer-events-none absolute left-0.5 inline-block h-6 w-6 rounded-full bg-foreground shadow-lg",
          "transition-all duration-300",
          isAnimating && "scale-90",
          checked 
            ? "translate-x-5" 
            : "translate-x-0"
        )}
        style={{
          transitionTimingFunction: isAnimating 
            ? "cubic-bezier(0.34, 1.56, 0.64, 1)" 
            : "cubic-bezier(0.16, 1, 0.3, 1)",
        }}
      />
    </button>
  );
};
