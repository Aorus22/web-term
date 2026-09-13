import * as React from "react"

import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { XIcon } from "lucide-react"

/**
 * Push-aside side panel (wa-bot style sheet): rendered inline as a flex
 * sibling so it squeezes the content beside it instead of overlaying it.
 *
 * The outer slot animates its width (0 ↔ 540px) while the inner panel keeps
 * its full width and is clipped, so the neighbouring content narrows smoothly
 * instead of being covered by a fixed overlay with a backdrop.
 */
const PANEL_WIDTH_PX = 540
const PANEL_ANIM_MS = 240

function SidePanel({
  open,
  onOpenChange,
  children,
}: {
  open: boolean
  onOpenChange?: (open: boolean) => void
  children: React.ReactNode
}) {
  const [mounted, setMounted] = React.useState(open)
  const [expanded, setExpanded] = React.useState(false)

  React.useEffect(() => {
    if (open) {
      setMounted(true)
      // Double rAF so the w-0 → w-[540px] transition plays on mount.
      const raf = requestAnimationFrame(() =>
        requestAnimationFrame(() => setExpanded(true))
      )
      return () => cancelAnimationFrame(raf)
    }
    setExpanded(false)
    const t = setTimeout(() => setMounted(false), PANEL_ANIM_MS)
    return () => clearTimeout(t)
  }, [open])

  if (!mounted) return null

  return (
    <div
      data-slot="side-panel-slot"
      aria-hidden={!open}
      className={cn(
        "h-full shrink-0 overflow-hidden transition-[width] ease-out duration-[240ms]",
        expanded ? "w-[540px]" : "w-0"
      )}
    >
      <div
        data-slot="side-panel-content"
        style={{ width: PANEL_WIDTH_PX }}
        className="relative flex h-full flex-col bg-popover text-popover-foreground shadow-lg border-l"
      >
        {onOpenChange && (
          <Button
            variant="ghost"
            className="absolute top-3 right-3"
            size="icon-sm"
            onClick={() => onOpenChange(false)}
          >
            <XIcon />
            <span className="sr-only">Close</span>
          </Button>
        )}
        {children}
      </div>
    </div>
  )
}

function SidePanelContent({ children }: { children: React.ReactNode }) {
  return <>{children}</>
}

function SidePanelHeader({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="side-panel-header"
      className={cn("relative flex flex-col gap-0.5 p-4 pr-12", className)}
      {...props}
    />
  )
}

function SidePanelTitle({ className, ...props }: React.ComponentProps<"h2">) {
  return (
    <h2
      data-slot="side-panel-title"
      className={cn("font-heading text-base font-medium text-foreground", className)}
      {...props}
    />
  )
}

function SidePanelDescription({
  className,
  ...props
}: React.ComponentProps<"p">) {
  return (
    <p
      data-slot="side-panel-description"
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  )
}

function SidePanelFooter({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="side-panel-footer"
      className={cn("mt-auto flex flex-col gap-2 p-4", className)}
      {...props}
    />
  )
}

export {
  SidePanel,
  SidePanelContent,
  SidePanelHeader,
  SidePanelTitle,
  SidePanelDescription,
  SidePanelFooter,
}
