<script lang="ts">
  import type { Course, CourseProgress, CourseStats, Lecture, Section } from '../types';
  import { formatTime, formatTotalDuration } from '../utils';
  import Button from './ui/Button.svelte';
  import Input from './ui/Input.svelte';
  import {
    ChevronDown,
    ChevronUp,
    Check,
    Play,
    Search,
    X,
    FolderKanban,
  } from 'lucide-svelte';
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    course: Course | null;
    stats: CourseStats | null;
    progress: CourseProgress | null;
    activeLecture: Lecture | null;
    isOpen: boolean;
    onSelectLecture: (lec: Lecture, sec: Section) => void;
    onToggleCompleted: (lectureId: string, event: MouseEvent) => void;
    onCloseSidebar: () => void;
  }

  let {
    course,
    stats,
    progress,
    activeLecture,
    isOpen,
    onSelectLecture,
    onToggleCompleted,
    onCloseSidebar,
  }: Props = $props();

  let searchQuery = $state('');
  let expandedSections: Record<string, boolean> = $state({});
  let allExpanded = $state(false);

  $effect(() => {
    if (course) {
      const initial: Record<string, boolean> = {};
      course.sections.forEach((s) => {
        // Expand only the section that contains the currently active lecture
        const containsActive = activeLecture ? s.lectures.some((l) => l.id === activeLecture.id) : false;
        initial[s.id] = containsActive;
      });
      expandedSections = initial;
    }
  });

  function toggleSection(secId: string) {
    expandedSections[secId] = !expandedSections[secId];
  }

  function toggleAll() {
    allExpanded = !allExpanded;
    const updated: Record<string, boolean> = {};
    if (course) {
      course.sections.forEach((s) => {
        updated[s.id] = allExpanded;
      });
    }
    expandedSections = updated;
  }
</script>

<aside
  class="bg-card border-l border-border flex flex-col shrink-0 h-full overflow-hidden transition-all duration-300 ease-out z-30 select-none {isOpen
    ? 'w-[380px] opacity-100 shadow-xl'
    : 'w-0 border-l-0 opacity-0 pointer-events-none'}"
>
  <!-- Header -->
  <div class="p-4 border-b border-border flex items-center justify-between shrink-0 bg-secondary/90 backdrop-blur-md">
    <div class="flex items-center gap-2">
      <FolderKanban class="w-4 h-4 text-[#a435f0]" />
      <span class="font-extrabold text-sm text-foreground tracking-tight">Course content</span>
    </div>

    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        class="w-7 h-7 rounded text-muted-foreground hover:text-foreground"
        onclick={toggleAll}
        title="Expand / Collapse all sections"
      >
        {#if allExpanded}
          <ChevronUp class="w-4 h-4 transition-transform duration-200" />
        {:else}
          <ChevronDown class="w-4 h-4 transition-transform duration-200" />
        {/if}
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="w-7 h-7 rounded text-muted-foreground hover:text-foreground"
        onclick={onCloseSidebar}
        title="Close sidebar"
      >
        <X class="w-4 h-4" />
      </Button>
    </div>
  </div>

  <!-- Search Filter -->
  <div class="p-3 border-b border-border bg-card/70">
    <div class="relative flex items-center">
      <Search class="w-3.5 h-3.5 absolute left-3 text-muted-foreground pointer-events-none" />
      <Input
        type="text"
        placeholder="Search lectures in course..."
        class="pl-9 h-8 text-xs bg-secondary/80 focus:bg-background transition-all duration-200"
        bind:value={searchQuery}
      />
    </div>
  </div>

  <!-- Scrollable Sections List -->
  <div class="flex-1 overflow-y-auto divide-y divide-border/80">
    {#if course && course.sections.length > 0}
      {#each course.sections as sec (sec.id)}
        {@const completedInSec = sec.lectures.filter((l) =>
          progress?.completed_lecture_ids?.includes(l.id)
        ).length}
        {@const secTotalDuration = sec.lectures.reduce(
          (acc, l) => acc + (l.duration_seconds || 0),
          0
        )}
        {@const isExpanded = expandedSections[sec.id] ?? true}
        {@const hasActiveLecture = sec.lectures.some((l) => activeLecture?.id === l.id)}
        {@const filteredLectures = sec.lectures.filter((l) =>
          !searchQuery.trim() || l.title.toLowerCase().includes(searchQuery.toLowerCase().trim())
        )}

        {#if searchQuery.trim() === '' || filteredLectures.length > 0}
          <div class="border-b border-border/80">
            
            <!-- Distinct Section Headline Header -->
            <button
              class="w-full text-left px-4 py-3.5 flex items-center justify-between gap-3 transition-all duration-200 cursor-pointer group border-l-4 {hasActiveLecture
                ? 'bg-[#2a2b2e] dark:bg-[#2d2f34] border-l-[#a435f0] shadow-xs'
                : 'bg-[#f0f2f5] dark:bg-[#25272a] border-l-transparent hover:bg-[#e4e7ec] dark:hover:bg-[#2c2e33]'}"
              onclick={() => toggleSection(sec.id)}
            >
              <div class="flex-1 min-w-0">
                <div class="font-extrabold text-[13px] text-foreground tracking-tight group-hover:text-[#a435f0] transition-colors truncate mb-1.5">
                  {sec.title}
                </div>
                <div class="flex items-center gap-2">
                  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-bold bg-background/80 text-muted-foreground border border-border/50 shadow-2xs">
                    {completedInSec}/{sec.lectures.length} completed
                  </span>
                  {#if secTotalDuration > 0}
                    <span class="text-[11px] text-muted-foreground font-medium">
                      • {formatTotalDuration(secTotalDuration)}
                    </span>
                  {/if}
                </div>
              </div>
              <div class="text-muted-foreground p-1 rounded-md group-hover:text-foreground transition-transform duration-300 ease-out {isExpanded ? 'rotate-180 text-foreground' : ''}">
                <ChevronDown class="w-4 h-4" />
              </div>
            </button>

            <!-- Lectures in Section (Clean Background) -->
            {#if isExpanded}
              <div
                transition:slide={{ duration: 250, easing: cubicOut }}
                class="bg-background divide-y divide-border/30 overflow-hidden"
              >
                {#each filteredLectures as lec (lec.id)}
                  {@const isCompleted = progress?.completed_lecture_ids?.includes(lec.id) ?? false}
                  {@const isActive = activeLecture?.id === lec.id}

                  <div
                    class="group flex items-start gap-3 px-4 py-2.5 cursor-pointer transition-all duration-200 relative {isActive
                      ? 'bg-[#a435f0]/12 text-[#a435f0] dark:text-[#c0c4fc]'
                      : 'hover:bg-secondary/40 text-foreground hover:translate-x-0.5'}"
                    onclick={() => onSelectLecture(lec, sec)}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => e.key === 'Enter' && onSelectLecture(lec, sec)}
                  >
                    {#if isActive}
                      <div class="absolute left-0 top-0 bottom-0 w-1 bg-[#a435f0] shadow-sm animate-pulse-subtle"></div>
                    {/if}

                    <!-- Checkbox Button with bounce animation -->
                    <button
                      class="w-4 h-4 rounded border mt-0.5 flex items-center justify-center shrink-0 transition-all duration-200 active:scale-75 {isCompleted
                        ? 'bg-[#a435f0] border-[#a435f0] text-white shadow-xs'
                        : 'border-border/90 hover:border-[#a435f0] hover:scale-110 bg-transparent'}"
                      onclick={(e) => onToggleCompleted(lec.id, e)}
                      title={isCompleted ? 'Mark incomplete' : 'Mark complete'}
                    >
                      {#if isCompleted}
                        <div class="animate-bounce-check">
                          <Check class="w-3 h-3 stroke-[3]" />
                        </div>
                      {/if}
                    </button>

                    <!-- Title & Duration -->
                    <div class="flex-1 min-w-0">
                      <div class="text-xs leading-snug font-medium truncate mb-1 transition-colors {isActive ? 'font-bold text-[#a435f0] dark:text-[#c0c4fc]' : 'group-hover:text-foreground text-foreground/90'}">
                        {lec.title}
                      </div>
                      <div class="flex items-center gap-1.5 text-[10px] text-muted-foreground">
                        <Play class="w-2.5 h-2.5 fill-current transition-transform duration-200 group-hover:scale-110 {isActive ? 'text-[#a435f0]' : ''}" />
                        <span>{lec.duration_seconds > 0 ? formatTime(lec.duration_seconds) : 'Video'}</span>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      {/each}
    {:else}
      <div class="p-8 text-center text-xs text-muted-foreground">
        No course content loaded.
      </div>
    {/if}
  </div>
</aside>
