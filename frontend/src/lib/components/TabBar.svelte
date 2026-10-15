<script lang="ts">
  import type { CourseTab } from '../types';
  import { Play, Code, FileText, X } from 'lucide-svelte';

  interface Props {
    tabs: CourseTab[];
    activeTabId: string;
    onSelectTab: (tabId: string) => void;
    onCloseTab: (tabId: string) => void;
  }

  let { tabs, activeTabId, onSelectTab, onCloseTab }: Props = $props();

  let containerElement: HTMLElement | null = $state(null);

  function handleWheel(e: WheelEvent) {
    if (e.deltaY !== 0 && containerElement) {
      e.preventDefault();
      containerElement.scrollLeft += e.deltaY;
    }
  }
</script>

{#if tabs.length > 0}
  <div
    bind:this={containerElement}
    onwheel={handleWheel}
    class="h-10 bg-[#121122] border-b border-border/80 flex items-center px-2.5 shrink-0 overflow-x-auto no-scrollbar gap-1.5 select-none z-20"
    role="tablist"
    aria-label="Open course tabs"
  >
    {#each tabs as tab (tab.id)}
      {@const isActive = tab.id === activeTabId}
      <button
        type="button"
        role="tab"
        aria-selected={isActive}
        class="group relative h-[30px] px-3 rounded-lg flex items-center gap-2 text-xs font-medium transition-all max-w-[240px] shrink-0 cursor-pointer border {isActive
          ? 'bg-[#22203d] text-white border-primary/50 shadow-sm shadow-primary/10'
          : 'bg-[#18172c]/80 hover:bg-[#1f1d38] text-[#9390ae] hover:text-[#e4e2f2] border-[#292745]/60 hover:border-[#38355a]'}"
        onclick={() => onSelectTab(tab.id)}
        onauxclick={(e) => {
          if (e.button === 1 && tab.closeable) {
            e.preventDefault();
            onCloseTab(tab.id);
          }
        }}
        title={tab.title}
      >
        <!-- Tab Type Icon -->
        <span class="shrink-0 flex items-center justify-center">
          {#if tab.type === 'video'}
            <Play class="w-3 h-3 fill-primary text-primary" />
          {:else if tab.type === 'html'}
            <Code class="w-3.5 h-3.5 text-amber-400" />
          {:else}
            <FileText class="w-3.5 h-3.5 text-rose-400" />
          {/if}
        </span>

        <!-- Tab Title -->
        <span class="truncate min-w-0 flex-1 text-left">
          {tab.title}
        </span>

        <!-- Close Button (only for closeable tabs) -->
        {#if tab.closeable}
          <span
            role="button"
            tabindex="0"
            class="shrink-0 w-4 h-4 rounded flex items-center justify-center text-[#827f9e] hover:text-white hover:bg-white/10 transition-colors ml-0.5"
            onclick={(e) => {
              e.stopPropagation();
              onCloseTab(tab.id);
            }}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.stopPropagation();
                onCloseTab(tab.id);
              }
            }}
            title="Close tab"
            aria-label="Close tab"
          >
            <X class="w-3 h-3 stroke-[2.5]" />
          </span>
        {/if}

        <!-- Active Bottom Highlight Bar -->
        {#if isActive}
          <div class="absolute -bottom-[1px] left-3 right-3 h-[2px] bg-primary rounded-full shadow-[0_0_8px_rgba(139,92,246,0.6)]"></div>
        {/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
