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
        "relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full",
        "transition-colors duration-200 ease-out-expo",
        "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-background",
        "disabled:cursor-not-allowed disabled:opacity-50",
        checked 
          ? "bg-primary/70" 
          : "bg-muted",
        className
      )}
    >
      {/* Thumb */}
      <span
        className={cn(
          "pointer-events-none absolute left-0.5 inline-block h-4 w-4 rounded-full bg-foreground shadow",
          "transition-all duration-200",
          isAnimating && "scale-90",
          checked 
            ? "translate-x-4" 
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
