<script lang="ts">
  import { cn } from '../../utils';
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link' | 'purple';
    size?: 'default' | 'sm' | 'lg' | 'icon';
    children?: Snippet;
    class?: string;
  }

  let {
    variant = 'default',
    size = 'default',
    class: className = '',
    children,
    ...restProps
  }: Props = $props();

  const variants = {
    default: 'bg-primary text-primary-foreground shadow-xs hover:opacity-90 active:scale-[0.98]',
    purple: 'bg-primary text-primary-foreground hover:bg-[#5a32cf] font-semibold active:scale-[0.98]',
    destructive: 'bg-destructive text-destructive-foreground shadow-xs hover:opacity-90 active:scale-[0.98]',
    outline: 'border border-border bg-card shadow-xs hover:bg-secondary hover:text-foreground active:scale-[0.98]',
    secondary: 'bg-secondary text-secondary-foreground shadow-xs hover:bg-[#2c294a] active:scale-[0.98]',
    ghost: 'hover:bg-secondary hover:text-foreground active:scale-[0.96]',
    link: 'text-primary underline-offset-4 hover:underline',
  };

  const sizes = {
    default: 'h-9 px-4 py-2 text-sm',
    sm: 'h-8 rounded-md px-3 text-xs',
    lg: 'h-10 rounded-md px-8 text-base',
    icon: 'h-9 w-9 p-0',
  };
</script>

<button
  class={cn(
    'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md font-medium transition-all duration-200 ease-out focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 cursor-pointer select-none',
    variants[variant],
    sizes[size],
    className
  )}
  {...restProps}
>
  {@render children?.()}
</button>
