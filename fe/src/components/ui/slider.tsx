"use client"

import * as React from "react"
import { Slider as SliderPrimitive } from "@base-ui/react/slider"

import { cn } from "@/lib/utils"

function Slider({
  className,
  defaultValue,
  value,
  onValueChange,
  min = 0,
  max = 100,
  step = 1,
  ...props
}: Omit<SliderPrimitive.Root.Props, "value" | "onValueChange" | "defaultValue"> & {
  value?: number
  defaultValue?: number
  onValueChange?: (value: number) => void
}) {
  const handleValueChange = React.useCallback(
    (values: number[]) => {
      onValueChange?.(values[0])
    },
    [onValueChange]
  )

  const internalValue = React.useMemo(() => (value !== undefined ? [value] : undefined), [value])
  const internalDefaultValue = React.useMemo(
    () => (defaultValue !== undefined ? [defaultValue] : undefined),
    [defaultValue]
  )

  return (
    <SliderPrimitive.Root
      data-slot="slider"
      value={internalValue}
      defaultValue={internalDefaultValue}
      onValueChange={handleValueChange}
      min={min}
      max={max}
      step={step}
      className={cn(
        "relative flex w-full touch-none select-none items-center",
        className
      )}
      {...props}
    >
      <SliderPrimitive.Control
        data-slot="slider-control"
        className="relative flex w-full items-center py-3"
      >
        <SliderPrimitive.Track
          data-slot="slider-track"
          className="bg-muted relative h-1 w-full grow overflow-hidden rounded-full"
        >
          <SliderPrimitive.Indicator
            data-slot="slider-indicator"
            className="bg-primary absolute h-full rounded-full"
          />
        </SliderPrimitive.Track>
        <SliderPrimitive.Thumb
          data-slot="slider-thumb"
          className="bg-background ring-ring/50 block size-4.5 rounded-full border border-foreground/10 shadow-md transition-all outline-none hover:scale-110 active:scale-95 focus-visible:ring-3 disabled:pointer-events-none disabled:opacity-50 cursor-grab active:cursor-grabbing"
        />
      </SliderPrimitive.Control>
    </SliderPrimitive.Root>
  )
}

export { Slider }
