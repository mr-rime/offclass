<script lang="ts">
  import { cn } from '../../utils';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  interface Props extends HTMLAttributes<HTMLDivElement> {
    children?: Snippet;
    class?: string;
  }

  let { class: className = '', children, ...restProps }: Props = $props();
</script>

<div
  class={cn('rounded-xl border bg-card text-card-foreground shadow', className)}
  {...restProps}
>
  {@render children?.()}
</div>
