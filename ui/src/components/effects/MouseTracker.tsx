import { createSignal, createEffect, onCleanup, ParentComponent, JSX } from 'solid-js';
import { cn } from '../../lib/utils';

// Spotlight effect that follows mouse
export const SpotlightCard: ParentComponent<{
  class?: string;
  spotlightColor?: string;
  children: JSX.Element;
}> = (props) => {
  const [ref, setRef] = createSignal<HTMLDivElement>();
  const [mousePosition, setMousePosition] = createSignal({ x: 0, y: 0 });
  const [isHovering, setIsHovering] = createSignal(false);

  const handleMouseMove = (e: MouseEvent) => {
    const el = ref();
    if (!el) return;
    
    const rect = el.getBoundingClientRect();
    setMousePosition({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    });
  };

  return (
    <div
      ref={setRef}
      class={cn(
        'relative overflow-hidden',
        props.class
      )}
      onMouseMove={handleMouseMove}
      onMouseEnter={() => setIsHovering(true)}
      onMouseLeave={() => setIsHovering(false)}
    >
      {/* Spotlight gradient */}
      <div
        class="pointer-events-none absolute -inset-px transition-opacity duration-300"
        style={{
          background: `radial-gradient(600px circle at ${mousePosition().x}px ${mousePosition().y}px, ${props.spotlightColor || 'rgba(245, 158, 11, 0.1)'}, transparent 40%)`,
          opacity: isHovering() ? 1 : 0,
        }}
      />
      {props.children}
    </div>
  );
};

// Magnetic button that follows mouse slightly
export const MagneticButton: ParentComponent<{
  class?: string;
  strength?: number;
  onClick?: () => void;
}> = (props) => {
  const [ref, setRef] = createSignal<HTMLButtonElement>();
  const [position, setPosition] = createSignal({ x: 0, y: 0 });
  const strength = props.strength || 0.3;

  const handleMouseMove = (e: MouseEvent) => {
    const el = ref();
    if (!el) return;

    const rect = el.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    
    const distanceX = (e.clientX - centerX) * strength;
    const distanceY = (e.clientY - centerY) * strength;

    setPosition({ x: distanceX, y: distanceY });
  };

  const handleMouseLeave = () => {
    setPosition({ x: 0, y: 0 });
  };

  return (
    <button
      ref={setRef}
      class={cn(
        'transition-transform duration-100 ease-out',
        props.class
      )}
      style={{
        transform: `translate(${position().x}px, ${position().y}px)`,
      }}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      onClick={props.onClick}
    >
      {props.children}
    </button>
  );
};

// Tilt effect card
export const TiltCard: ParentComponent<{
  class?: string;
  maxTilt?: number;
  children: JSX.Element;
}> = (props) => {
  const [ref, setRef] = createSignal<HTMLDivElement>();
  const [transform, setTransform] = createSignal('');
  const [glarePosition, setGlarePosition] = createSignal({ x: 50, y: 50 });
  const maxTilt = props.maxTilt || 10;

  const handleMouseMove = (e: MouseEvent) => {
    const el = ref();
    if (!el) return;

    const rect = el.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    
    const percentX = (e.clientX - centerX) / (rect.width / 2);
    const percentY = (e.clientY - centerY) / (rect.height / 2);

    const rotateX = -percentY * maxTilt;
    const rotateY = percentX * maxTilt;

    setTransform(`perspective(1000px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale3d(1.02, 1.02, 1.02)`);
    
    // Update glare position
    setGlarePosition({
      x: ((e.clientX - rect.left) / rect.width) * 100,
      y: ((e.clientY - rect.top) / rect.height) * 100,
    });
  };

  const handleMouseLeave = () => {
    setTransform('perspective(1000px) rotateX(0) rotateY(0) scale3d(1, 1, 1)');
  };

  return (
    <div
      ref={setRef}
      class={cn(
        'relative transition-transform duration-200 ease-out',
        props.class
      )}
      style={{ transform: transform() }}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
    >
      {props.children}
      {/* Glare effect */}
      <div
        class="pointer-events-none absolute inset-0 opacity-0 hover:opacity-100 transition-opacity duration-300"
        style={{
          background: `radial-gradient(circle at ${glarePosition().x}% ${glarePosition().y}%, rgba(255,255,255,0.1) 0%, transparent 60%)`,
        }}
      />
    </div>
  );
};

// Mouse position display (for debugging or visual effects)
export function useMousePosition() {
  const [position, setPosition] = createSignal({ x: 0, y: 0 });

  createEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      setPosition({ x: e.clientX, y: e.clientY });
    };

    window.addEventListener('mousemove', handleMouseMove);
    onCleanup(() => window.removeEventListener('mousemove', handleMouseMove));
  });

  return position;
}
