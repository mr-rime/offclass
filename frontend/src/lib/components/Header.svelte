<script lang="ts">
  import type { Course, CourseStats, CourseProgress } from '../types';
  import Button from './ui/Button.svelte';
  import Progress from './ui/Progress.svelte';
  import {
    FolderPlus,
    Sun,
    Moon,
    PanelRight,
    CheckCircle2,
    Layers,
    Play,
    GraduationCap,
  } from 'lucide-svelte';
  import { fly, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    activeView: 'player' | 'library';
    currentCourse: Course | null;
    stats: CourseStats | null;
    progress: CourseProgress | null;
    theme: 'dark' | 'light';
    sidebarOpen: boolean;
    onToggleTheme: () => void;
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
    theme,
    sidebarOpen,
    onToggleTheme,
    onToggleSidebar,
    onNavigateLibrary,
    onNavigatePlayer,
    onImportCourse,
  }: Props = $props();

  let completedCount = $derived(
    progress?.completed_lecture_ids ? progress.completed_lecture_ids.length : 0
  );
  let totalCount = $derived(stats?.total_lectures ?? 0);
  let progressPct = $derived(
    totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0
  );
</script>

<header
  class="h-[60px] bg-secondary/80 backdrop-blur-md border-b border-border flex items-center justify-between px-5 z-40 shrink-0 select-none shadow-xs transition-all duration-300"
>
  <!-- Left Brand & Navigation -->
  <div class="flex items-center gap-4 flex-1 min-w-0">
    <button
      class="group flex items-center gap-2.5 cursor-pointer font-bold text-base tracking-tight hover:opacity-90 active:scale-95 transition-all duration-200"
      onclick={onNavigateLibrary}
      title="Go to My Courses page"
    >
      <span
        class="w-7 h-7 rounded-lg bg-gradient-to-tr from-[#a435f0] to-[#5624d0] text-white flex items-center justify-center shadow-sm group-hover:shadow-[#a435f0]/40 group-hover:shadow-md transition-all duration-300 group-hover:scale-105"
      >
        <GraduationCap class="w-4 h-4" />
      </span>
      <span class="font-extrabold text-lg text-foreground group-hover:text-[#a435f0] transition-colors tracking-tight">
        OffClass
      </span>
    </button>

    <div class="h-5 w-[1px] bg-border transition-colors"></div>

    {#if activeView === 'player' && currentCourse}
      <button
        class="font-semibold text-sm text-foreground truncate max-w-[420px] transition-all duration-200 hover:text-[#a435f0] cursor-pointer text-left"
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
      <span class="font-bold text-sm text-muted-foreground">
        My learning library
      </span>
    {/if}
  </div>

  <!-- Center Progress Pill (in player view) -->
  {#if activeView === 'player' && currentCourse && stats}
    <div in:scale={{ start: 0.9, duration: 250, easing: cubicOut }}>
      <button
        class="group flex items-center gap-3 bg-card px-3.5 py-1.5 rounded-full border border-border/80 shadow-xs hover:border-[#a435f0] hover:shadow-[#a435f0]/15 hover:shadow-md transition-all duration-200 active:scale-95 cursor-pointer"
        onclick={onNavigateLibrary}
        title="Course progress (Click to view courses)"
      >
        <CheckCircle2 class="w-4 h-4 text-[#a435f0] group-hover:rotate-12 transition-transform duration-300" />
        <span class="text-xs font-bold text-foreground">
          {completedCount} / {totalCount} ({progressPct}%)
        </span>
        <div class="w-24">
          <Progress value={progressPct} class="h-1.5" />
        </div>
      </button>
    </div>
  {/if}

  <!-- Right Actions -->
  <div class="flex items-center gap-2.5">
    {#if activeView === 'player'}
      <Button
        variant="secondary"
        size="sm"
        class="gap-1.5 text-xs font-semibold"
        onclick={onNavigateLibrary}
      >
        <Layers class="w-3.5 h-3.5" />
        <span>My Courses</span>
      </Button>
    {:else if currentCourse}
      <Button
        variant="secondary"
        size="sm"
        class="gap-1.5 text-xs font-semibold"
        onclick={onNavigatePlayer}
      >
        <Play class="w-3.5 h-3.5 fill-current text-[#a435f0]" />
        <span>Back to Player</span>
      </Button>
    {/if}

    <Button
      variant="purple"
      size="sm"
      class="gap-1.5 text-xs font-semibold"
      onclick={onImportCourse}
    >
      <FolderPlus class="w-3.5 h-3.5" />
      <span>Import Course</span>
    </Button>

    <Button
      variant="ghost"
      size="icon"
      class="rounded-full w-8 h-8 text-muted-foreground hover:text-foreground relative overflow-hidden"
      onclick={onToggleTheme}
      title="Toggle Dark / Light Mode"
    >
      <div class="transition-transform duration-500 {theme === 'dark' ? 'rotate-0' : 'rotate-180'}">
        {#if theme === 'dark'}
          <Sun class="w-4 h-4 text-amber-400" />
        {:else}
          <Moon class="w-4 h-4 text-slate-700" />
        {/if}
      </div>
    </Button>

    {#if activeView === 'player'}
      <Button
        variant={sidebarOpen ? 'secondary' : 'ghost'}
        size="icon"
        class="rounded-md w-8 h-8 text-muted-foreground hover:text-foreground transition-all duration-200"
        onclick={onToggleSidebar}
        title="Toggle Course Content Sidebar"
      >
        <PanelRight class="w-4 h-4 transition-transform duration-300 {sidebarOpen ? 'scale-105 text-foreground' : ''}" />
      </Button>
    {/if}
  </div>
</header>
