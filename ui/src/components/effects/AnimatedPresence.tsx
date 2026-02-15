import { ParentComponent, children, createEffect, onCleanup, createSignal } from 'solid-js';
import { createPresence } from '@solid-primitives/presence';
import { cn } from '../../lib/utils';

interface AnimatedPresenceProps {
  show: boolean;
  duration?: number;
  enterClass?: string;
  exitClass?: string;
  enterActiveClass?: string;
  exitActiveClass?: string;
  class?: string;
  onExit?: () => void;
}

export const AnimatedPresence: ParentComponent<AnimatedPresenceProps> = (props) => {
  const resolved = children(() => props.children);
  const duration = props.duration || 300;
  
  const { isMounted, isVisible } = createPresence(() => props.show, {
    transitionDuration: duration,
  });

  return (
    <div
      class={cn(
        'transition-all',
        props.class,
        isVisible() ? props.enterActiveClass : props.exitActiveClass
      )}
      style={{
        opacity: isVisible() ? 1 : 0,
        transform: isVisible() ? 'translateY(0)' : 'translateY(-10px)',
        transition: `opacity ${duration}ms ease-out, transform ${duration}ms ease-out`,
      }}
    >
      {isMounted() && resolved()}
    </div>
  );
};

// Fade in from bottom animation wrapper
export const FadeInUp: ParentComponent<{ delay?: number; class?: string }> = (props) => {
  const [ref, setRef] = createSignal<HTMLDivElement>();
  const [isVisible, setIsVisible] = createSignal(false);

  createEffect(() => {
    const el = ref();
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setTimeout(() => setIsVisible(true), props.delay || 0);
          observer.unobserve(el);
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(el);
    onCleanup(() => observer.disconnect());
  });

  return (
    <div
      ref={setRef}
      class={cn(
        'transition-all duration-500 ease-out',
        props.class,
        isVisible() 
          ? 'opacity-100 translate-y-0' 
          : 'opacity-0 translate-y-4'
      )}
    >
      {props.children}
    </div>
  );
};

// Pulse glow effect for active elements
export const PulseGlow: ParentComponent<{ 
  color?: 'amber' | 'emerald' | 'rose' | 'indigo' | 'cyan';
  intensity?: 'low' | 'medium' | 'high';
  class?: string;
}> = (props) => {
  const colorMap = {
    amber: 'shadow-amber-500/50',
    emerald: 'shadow-emerald-500/50',
    rose: 'shadow-rose-500/50',
    indigo: 'shadow-indigo-500/50',
    cyan: 'shadow-cyan-500/50',
  };

  const intensityMap = {
    low: 'shadow-[0_0_10px]',
    medium: 'shadow-[0_0_20px]',
    high: 'shadow-[0_0_40px]',
  };

  return (
    <div 
      class={cn(
        'animate-pulse-glow',
        colorMap[props.color || 'amber'],
        intensityMap[props.intensity || 'medium'],
        props.class
      )}
    >
      {props.children}
    </div>
  );
};

// Typing text effect
export const TypewriterText: ParentComponent<{
  text: string;
  speed?: number;
  class?: string;
  onComplete?: () => void;
}> = (props) => {
  const [displayText, setDisplayText] = createSignal('');
  const [isComplete, setIsComplete] = createSignal(false);

  createEffect(() => {
    setDisplayText('');
    setIsComplete(false);
    let currentIndex = 0;
    const text = props.text;
    const speed = props.speed || 30;

    const interval = setInterval(() => {
      if (currentIndex < text.length) {
        setDisplayText(text.slice(0, currentIndex + 1));
        currentIndex++;
      } else {
        clearInterval(interval);
        setIsComplete(true);
        props.onComplete?.();
      }
    }, speed);

    onCleanup(() => clearInterval(interval));
  });

  return (
    <span class={cn('font-mono', props.class)}>
      {displayText()}
      {!isComplete() && (
        <span class="animate-blink">_</span>
      )}
    </span>
  );
};
