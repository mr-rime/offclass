<script lang="ts">
  import { cn } from '../../utils';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  interface Props extends HTMLAttributes<HTMLDivElement> {
    variant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'purple';
    children?: Snippet;
    class?: string;
  }

  let {
    variant = 'default',
    class: className = '',
    children,
    ...restProps
  }: Props = $props();

  const variants = {
    default: 'border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80',
    secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
    destructive: 'border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80',
    outline: 'text-foreground',
    purple: 'border-transparent bg-[#a435f0]/15 text-[#a435f0] dark:text-[#c0c4fc] font-semibold',
  };
</script>

<div
  class={cn(
    'inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
    variants[variant],
    className
  )}
  {...restProps}
>
  {@render children?.()}
</div>
