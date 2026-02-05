import React, { useState, useRef, useEffect } from 'react'
import { motion } from 'framer-motion'
import { Button } from './button';
import { cn } from '@/lib/utils';

type Props = {
    icon: React.ReactNode;
    label: string;
    onClick?: () => void;
    className?: string;
    disabled?: boolean;
    loading?: boolean;
    baseSize: "xsmall" | "small" | "large";
    variant?: "default" | "outline" | "ghost" | "link";
    /** Direction the button expands. Use "left" when button is on the right side of its container */
    expandDirection?: "left" | "right";
}

export default function SizeableButton({
  icon,
  label,
  onClick,
  className,
  disabled,
  loading,
  baseSize,
  variant = "default",
  expandDirection = "left"
}: Props) {
  const [isHovered, setIsHovered] = useState(false);
  const [labelWidth, setLabelWidth] = useState(0);
  const measureRef = useRef<HTMLSpanElement>(null);

  const iconSizePx = baseSize === "xsmall" ? 24 : baseSize === "small" ? 36 : 40;
  const gap = 8;
  const paddingLabel = 12;

  // Measure label width using hidden element
  useEffect(() => {
    if (measureRef.current) {
      setLabelWidth(measureRef.current.offsetWidth);
    }
  }, [label]);

  const collapsedWidth = iconSizePx;
  const expandedWidth = iconSizePx + gap + labelWidth + paddingLabel;
  const expandsLeft = expandDirection === "left";

  return (
    // Wrapper maintains fixed layout size (collapsed width)
    <div 
      className="relative m-0 p-0"
      style={{ width: collapsedWidth, height: iconSizePx }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Hidden element to measure label width */}
      <span
        ref={measureRef}
        aria-hidden="true"
        className="fixed opacity-0 pointer-events-none whitespace-nowrap text-sm"
        style={{ left: -10000, top: -10000 }}
      >
        {label}
      </span>

      {/* Animated button - positioned absolutely to not affect layout */}
      <motion.div
        className={cn(
          "absolute top-0",
          expandsLeft ? "right-0" : "left-0"
        )}
        initial={false}
        animate={{
          width: isHovered ? expandedWidth : collapsedWidth,
        }}
        transition={{
          type: "spring",
          stiffness: 400,
          damping: 32,
        }}
      >
        <Button
          onClick={onClick}
          disabled={disabled || loading}
          variant={variant}
          size={baseSize === "xsmall" ? "icon" : baseSize === "small" ? "sm" : "default"}
          className={cn(
            "w-full overflow-hidden px-0",
            className
          )}
          style={{ height: iconSizePx }}
        >
          <div className={cn(
            "flex items-center whitespace-nowrap",
            expandsLeft && "flex-row-reverse"
          )}>
            {/* Icon container - fixed size */}
            <div 
              className="flex items-center justify-center shrink-0"
              style={{ width: iconSizePx, height: iconSizePx }}
            >
              {icon}
            </div>
            
            {/* Label with fade animation */}
            <motion.span
              className="overflow-hidden"
              initial={false}
              animate={{
                opacity: isHovered ? 1 : 0,
                width: isHovered ? labelWidth + paddingLabel : 0,
              }}
              transition={{
                opacity: { duration: 0.15 },
                width: { type: "spring", stiffness: 400, damping: 28 },
              }}
            >
              <span className={cn(expandsLeft ? "pl-3" : "pr-3")}>{label}</span>
            </motion.span>
          </div>
        </Button>
      </motion.div>
    </div>
  )
}