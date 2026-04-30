import { useEffect, useState } from 'react';
import { cn } from '@/lib/utils';

export function CustomCursor() {
  const [position, setPosition] = useState({ x: -100, y: -100 });
  const [isPointer, setIsPointer] = useState(false);
  const [isHidden, setIsHidden] = useState(true);
  const [isMouseDown, setIsMouseDown] = useState(false);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      setPosition({ x: e.clientX, y: e.clientY });
      setIsHidden(false);

      const target = e.target as HTMLElement;
      const isClickable = 
        window.getComputedStyle(target).cursor === 'pointer' ||
        target.tagName === 'BUTTON' ||
        target.tagName === 'A' ||
        target.closest('button') ||
        target.closest('a');
      
      setIsPointer(!!isClickable);
    };

    const onMouseLeave = () => setIsHidden(true);
    const onMouseEnter = () => setIsHidden(false);
    const onMouseDown = () => setIsMouseDown(true);
    const onMouseUp = () => setIsMouseDown(false);

    window.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseleave', onMouseLeave);
    document.addEventListener('mouseenter', onMouseEnter);
    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('mouseup', onMouseUp);

    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseleave', onMouseLeave);
      document.removeEventListener('mouseenter', onMouseEnter);
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }, []);

  if (isHidden) return null;

  return (
    <>
      {/* Main Cursor Dot */}
      <div 
        className={cn(
          "fixed pointer-events-none z-[9999] transition-transform duration-100 ease-out mix-blend-difference",
          isMouseDown ? "scale-75" : "scale-100"
        )}
        style={{
          left: position.x,
          top: position.y,
          transform: `translate(-50%, -50%) ${isMouseDown ? 'scale(0.8)' : 'scale(1)'}`,
        }}
      >
        <div className={cn(
          "w-2 h-2 rounded-full bg-white transition-all duration-300",
          isPointer && "w-6 h-6 border-2 border-white bg-transparent opacity-50"
        )} />
      </div>

      {/* Trailing Ring */}
      <div 
        className={cn(
          "fixed pointer-events-none z-[9998] transition-all duration-300 ease-out border border-white/30 rounded-full mix-blend-difference",
          isPointer ? "w-10 h-10 opacity-0" : "w-6 h-6 opacity-100"
        )}
        style={{
          left: position.x,
          top: position.y,
          transform: 'translate(-50%, -50%)',
          transition: 'width 0.3s, height 0.3s, opacity 0.3s, transform 0.15s ease-out',
        }}
      />
    </>
  );
}
