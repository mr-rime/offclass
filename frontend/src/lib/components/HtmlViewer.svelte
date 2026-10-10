<script lang="ts">
  import type { Course, Lecture, Section } from '../types';
  import Button from './ui/Button.svelte';
  import {
    Code,
    RefreshCw,
    Maximize,
    Minimize,
    ExternalLink,
    Check,
    SkipForward,
    Loader2,
  } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';

  interface Props {
    currentCourse: Course | null;
    activeLecture: Lecture | null;
    activeSection: Section | null;
    streamingPort: number;
    isCompleted?: boolean;
    onToggleCompleted?: (event: MouseEvent) => void;
    onNextLecture?: () => void;
  }

  let {
    currentCourse,
    activeLecture,
    activeSection,
    streamingPort,
    isCompleted = false,
    onToggleCompleted,
    onNextLecture,
  }: Props = $props();

  let iframeElement: HTMLIFrameElement | null = $state(null);
  let containerElement: HTMLElement | null = $state(null);
  let isFullscreen = $state(false);
  let isLoading = $state(true);

  let htmlUrl = $derived.by(() => {
    if (!currentCourse || !activeLecture || streamingPort <= 0) return '';
    if (activeLecture.relative_path) {
      const encodedRel = activeLecture.relative_path
        .replace(/\\/g, '/')
        .split('/')
        .map((p) => encodeURIComponent(p))
        .join('/');
      return `http://127.0.0.1:${streamingPort}/content/${currentCourse.id}/${encodedRel}`;
    }
    return `http://127.0.0.1:${streamingPort}/html/${currentCourse.id}/${activeLecture.id}`;
  });

  $effect(() => {
    if (activeLecture) {
      isLoading = true;
    }
  });

  function handleReload() {
    if (iframeElement && htmlUrl) {
      isLoading = true;
      iframeElement.src = htmlUrl;
    }
  }

  function handleIframeLoad() {
    isLoading = false;
  }

  async function toggleFullscreen() {
    if (!containerElement) return;
    try {
      if (!document.fullscreenElement) {
        await containerElement.requestFullscreen();
        isFullscreen = true;
      } else {
        await document.exitFullscreen();
        isFullscreen = false;
      }
    } catch (e) {
      console.warn('Fullscreen toggle failed:', e);
    }
  }

  function openInBrowser() {
    if (htmlUrl) {
      window.open(htmlUrl, '_blank');
    }
  }

  onMount(() => {
    const handleFullscreenChange = () => {
      isFullscreen = !!document.fullscreenElement;
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
    };
  });
</script>

<div
  bind:this={containerElement}
  class="w-full bg-[#0d0c18] flex flex-col min-h-[500px] h-[650px] max-h-[calc(100vh-140px)] shrink-0 overflow-hidden shadow-2xl select-none font-sans border-b border-border/80 {isFullscreen
    ? 'h-screen max-h-screen'
    : ''}"
  role="region"
  aria-label="HTML Document Viewer"
>
  <!-- Top Viewer Toolbar -->
  <div class="h-12 bg-[#161528] border-b border-border px-4 flex items-center justify-between shrink-0 z-20">
    <!-- Left: Title & Section -->
    <div class="flex items-center gap-2.5 min-w-0 pr-4">
      <div class="w-7 h-7 rounded-md bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-400 shrink-0">
        <Code class="w-4 h-4" />
      </div>

      <div class="flex items-center gap-2 min-w-0">
        <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-amber-500/10 text-amber-400 border border-amber-500/25 shrink-0">
          {activeSection?.title ?? 'Section'}
        </span>
        <h2 class="text-sm font-semibold text-foreground truncate" title={activeLecture?.title}>
          {activeLecture?.order}. {activeLecture?.title}
        </h2>
      </div>
    </div>

    <!-- Right: Actions Toolbar -->
    <div class="flex items-center gap-1.5 shrink-0">
      {#if onToggleCompleted && activeLecture}
        <Button
          variant={isCompleted ? 'purple' : 'secondary'}
          size="sm"
          class="h-7 px-2.5 text-xs gap-1.5 font-medium"
          onclick={(e) => onToggleCompleted(e)}
          title={isCompleted ? 'Mark as incomplete' : 'Mark as completed'}
        >
          <Check class="w-3.5 h-3.5 {isCompleted ? 'stroke-[3]' : ''}" />
          <span>{isCompleted ? 'Completed' : 'Mark Complete'}</span>
        </Button>
      {/if}

      {#if onNextLecture}
        <Button
          variant="secondary"
          size="sm"
          class="h-7 px-2.5 text-xs gap-1.5 text-muted-foreground hover:text-foreground"
          onclick={onNextLecture}
          title="Next Item"
        >
          <SkipForward class="w-3.5 h-3.5" />
          <span class="hidden sm:inline">Next</span>
        </Button>
      {/if}

      <div class="h-4 w-[1px] bg-border mx-1"></div>

      <Button
        variant="ghost"
        size="icon"
        class="w-7 h-7 rounded text-muted-foreground hover:text-foreground"
        onclick={handleReload}
        title="Reload document"
      >
        <RefreshCw class="w-3.5 h-3.5 {isLoading ? 'animate-spin text-primary' : ''}" />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        class="w-7 h-7 rounded text-muted-foreground hover:text-foreground"
        onclick={openInBrowser}
        title="Open in new window"
      >
        <ExternalLink class="w-3.5 h-3.5" />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        class="w-7 h-7 rounded text-muted-foreground hover:text-foreground"
        onclick={toggleFullscreen}
        title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
      >
        {#if isFullscreen}
          <Minimize class="w-3.5 h-3.5" />
        {:else}
          <Maximize class="w-3.5 h-3.5" />
        {/if}
      </Button>
    </div>
  </div>

  <!-- HTML Iframe Viewer Container -->
  <div class="relative flex-1 w-full h-full bg-white overflow-hidden">
    {#if isLoading}
      <div
        class="absolute inset-0 bg-[#121122] flex flex-col items-center justify-center gap-3 z-10"
        transition:fade={{ duration: 150 }}
      >
        <Loader2 class="w-8 h-8 text-amber-400 animate-spin" />
        <span class="text-xs font-medium text-muted-foreground">Rendering HTML page...</span>
      </div>
    {/if}

    {#if htmlUrl}
      <iframe
        bind:this={iframeElement}
        src={htmlUrl}
        title={activeLecture?.title ?? 'HTML Viewer'}
        class="w-full h-full border-0 bg-white"
        sandbox="allow-scripts allow-same-origin allow-popups allow-forms allow-downloads allow-modals"
        onload={handleIframeLoad}
        onerror={handleIframeLoad}
      ></iframe>
    {:else}
      <div class="h-full w-full bg-[#121122] flex flex-col items-center justify-center text-muted-foreground text-xs p-6 text-center">
        No HTML document loaded.
      </div>
    {/if}
  </div>
</div>
