<script lang="ts">
  import type { Course, CourseStats, CourseProgress } from '../types';
  import Button from './ui/Button.svelte';
  import Progress from './ui/Progress.svelte';
  import {
    FolderPlus,
    PanelRight,
    CheckCircle2,
    Layers,
    Play,
    GraduationCap,
    Loader2,
  } from 'lucide-svelte';
  import { fly, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    activeView: 'player' | 'library';
    currentCourse: Course | null;
    stats: CourseStats | null;
    progress: CourseProgress | null;
    sidebarOpen: boolean;
    isImporting?: boolean;
    onToggleSidebar: () => void;
    onNavigateLibrary: () => void;
    onNavigatePlayer: () => void;
    onImportCourse: () => void;
  }

  let {
    activeView,
    currentCourse,
    stats,
    progress,
    sidebarOpen,
    isImporting = false,
    onToggleSidebar,
    onNavigateLibrary,
    onNavigatePlayer,
    onImportCourse,
  }: Props = $props();

  let completedCount = $derived(
    stats?.completed_lectures ?? (progress?.completed_lecture_ids ? progress.completed_lecture_ids.length : 0)
  );
  let totalCount = $derived(stats?.total_lectures ?? 0);
  let progressPct = $derived(
    stats?.progress_percent != null
      ? Math.round(stats.progress_percent)
      : totalCount > 0
      ? Math.round((completedCount / totalCount) * 100)
      : 0
  );
</script>

<header
  class="h-[56px] bg-[#161528] border-b border-border flex items-center justify-between px-5 z-40 shrink-0 select-none transition-colors duration-200"
>
  <!-- Left Brand & Navigation -->
  <div class="flex items-center gap-4 flex-1 min-w-0">
    <button
      class="group flex items-center gap-2.5 cursor-pointer font-bold text-base tracking-tight hover:opacity-90 active:scale-95 transition-all duration-200"
      onclick={onNavigateLibrary}
      title="Go to My Courses page"
    >
      <span
        class="w-7 h-7 rounded-md bg-[#22203d] border border-border/80 flex items-center justify-center transition-all duration-200 group-hover:scale-105 overflow-hidden shadow-sm"
      >
        <img src="/logo.png" alt="OffClass" class="w-full h-full object-cover rounded-md" />
      </span>
      <span class="font-bold text-base text-foreground group-hover:text-primary transition-colors tracking-tight">
        OffClass
      </span>
    </button>

    <div class="h-4 w-[1px] bg-border transition-colors"></div>

    {#if activeView === 'player' && currentCourse}
      <button
        class="font-medium text-sm text-foreground truncate max-w-[420px] transition-all duration-200 hover:text-primary cursor-pointer text-left"
        onclick={onNavigateLibrary}
        title="{currentCourse.title} (Click to view all courses)"
      >
        {#key currentCourse.id}
          <span in:fly={{ y: -4, duration: 250, easing: cubicOut }}>
            {currentCourse.title}
          </span>
        {/key}
      </button>
    {:else}
      <span class="font-semibold text-xs text-muted-foreground uppercase tracking-wider">
        My learning library
      </span>
    {/if}
  </div>

  <!-- Center Progress Pill (in player view) -->
  {#if activeView === 'player' && currentCourse && stats}
    <div in:scale={{ start: 0.9, duration: 250, easing: cubicOut }}>
      <button
        class="group flex items-center gap-3 bg-card px-3.5 py-1.5 rounded-lg border border-border hover:border-primary transition-all duration-200 active:scale-95 cursor-pointer"
        onclick={onNavigateLibrary}
        title="Course progress (Click to view courses)"
      >
        <CheckCircle2 class="w-4 h-4 text-primary transition-transform duration-200 group-hover:scale-110" />
        <span class="text-xs font-semibold text-foreground">
          {completedCount} / {totalCount} ({progressPct}%)
        </span>
        <div class="w-24">
          <Progress value={progressPct} class="h-1.5" />
        </div>
      </button>
    </div>
  {/if}

  <!-- Right Actions -->
  <div class="flex items-center gap-2">
    {#if activeView === 'player'}
      <Button
        variant="secondary"
        size="sm"
        class="gap-1.5 text-xs font-medium"
        onclick={onNavigateLibrary}
      >
        <Layers class="w-3.5 h-3.5 text-muted-foreground" />
        <span>My Courses</span>
      </Button>
    {:else if currentCourse}
      <Button
        variant="secondary"
        size="sm"
        class="gap-1.5 text-xs font-medium"
        onclick={onNavigatePlayer}
      >
        <Play class="w-3.5 h-3.5 fill-current text-primary" />
        <span>Back to Player</span>
      </Button>
    {/if}

    <Button
      variant="purple"
      size="sm"
      class="gap-1.5 text-xs font-medium"
      onclick={onImportCourse}
      disabled={isImporting}
    >
      {#if isImporting}
        <Loader2 class="w-3.5 h-3.5 animate-spin" />
        <span>Importing...</span>
      {:else}
        <FolderPlus class="w-3.5 h-3.5" />
        <span>Import Course</span>
      {/if}
    </Button>

    {#if activeView === 'player'}
      <Button
        variant={sidebarOpen ? 'secondary' : 'ghost'}
        size="icon"
        class="rounded-md w-8 h-8 text-muted-foreground hover:text-foreground transition-all duration-200"
        onclick={onToggleSidebar}
        title="Toggle Course Content Sidebar"
      >
        <PanelRight class="w-4 h-4 transition-transform duration-200 {sidebarOpen ? 'text-foreground' : ''}" />
      </Button>
    {/if}
  </div>
</header>
