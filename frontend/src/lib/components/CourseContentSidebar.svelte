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
  class="bg-[#161528] border-l border-border flex flex-col shrink-0 h-full overflow-hidden transition-all duration-300 ease-out z-30 select-none {isOpen
    ? 'w-[380px] opacity-100'
    : 'w-0 border-l-0 opacity-0 pointer-events-none'}"
>
  <!-- Header -->
  <div class="p-3.5 border-b border-border flex items-center justify-between shrink-0 bg-[#161528]">
    <div class="flex items-center gap-2">
      <FolderKanban class="w-4 h-4 text-primary" />
      <span class="font-bold text-sm text-foreground tracking-tight">Course content</span>
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
  <div class="p-3 border-b border-border bg-[#18172c]">
    <div class="relative flex items-center">
      <Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none" />
      <Input
        type="text"
        placeholder="Search lectures in course..."
        class="pl-9 {searchQuery ? 'pr-8' : ''} h-8 text-xs bg-[#1e1c34] border-border focus:bg-[#22203d] transition-colors"
        bind:value={searchQuery}
      />
      {#if searchQuery}
        <button
          class="absolute right-2 top-1/2 -translate-y-1/2 w-5 h-5 rounded hover:bg-[#282548] flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
          onclick={() => (searchQuery = '')}
          title="Clear search"
        >
          <X class="w-3 h-3" />
        </button>
      {/if}
    </div>
  </div>

  <!-- Scrollable Sections List -->
  <div class="flex-1 overflow-y-auto divide-y divide-border/80 bg-[#161528]">
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
          <div class="border-b border-border">
            
            <!-- Section Headline Header -->
            <button
              class="w-full text-left px-4 py-3 flex items-center justify-between gap-3 transition-colors cursor-pointer group border-l-2 {hasActiveLecture
                ? 'bg-[#22203d] border-l-primary'
                : 'bg-[#1a1830] border-l-transparent hover:bg-[#201e38]'}"
              onclick={() => toggleSection(sec.id)}
            >
              <div class="flex-1 min-w-0">
                <div class="font-semibold text-[13px] text-foreground group-hover:text-primary transition-colors truncate mb-1">
                  {sec.title}
                </div>
                <div class="flex items-center gap-2">
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-medium bg-[#161528] text-muted-foreground border border-border">
                    {completedInSec}/{sec.lectures.length} completed
                  </span>
                  <span class="text-[11px] text-muted-foreground font-medium">
                    • {formatTotalDuration(secTotalDuration)}
                  </span>
                </div>
              </div>
              <div class="text-muted-foreground p-1 rounded-md group-hover:text-foreground transition-transform duration-200 {isExpanded ? 'rotate-180 text-foreground' : ''}">
                <ChevronDown class="w-4 h-4" />
              </div>
            </button>

            <!-- Lectures in Section -->
            {#if isExpanded}
              <div
                transition:slide={{ duration: 200, easing: cubicOut }}
                class="bg-[#161528] divide-y divide-border/40 overflow-hidden"
              >
                {#each filteredLectures as lec (lec.id)}
                  {@const isCompleted = progress?.completed_lecture_ids?.includes(lec.id) ?? false}
                  {@const isActive = activeLecture?.id === lec.id}

                  <div
                    class="group flex items-start gap-3 px-4 py-2.5 cursor-pointer transition-colors relative {isActive
                      ? 'bg-[#362857] text-white'
                      : 'hover:bg-[#201e38] text-[#8c88aa] hover:text-foreground'}"
                    onclick={() => onSelectLecture(lec, sec)}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => e.key === 'Enter' && onSelectLecture(lec, sec)}
                  >
                    {#if isActive}
                      <div class="absolute left-0 top-0 bottom-0 w-1 bg-primary"></div>
                    {/if}

                    <!-- Checkbox Button with distinct visible background -->
                    <button
                      class="w-[18px] h-[18px] rounded-[4px] mt-0.5 flex items-center justify-center shrink-0 transition-all cursor-pointer shadow-sm {isCompleted
                        ? 'bg-primary border border-primary text-white shadow-primary/25'
                        : 'bg-[#282548] border border-[#4a4576] hover:bg-[#34305c] hover:border-primary text-transparent'}"
                      onclick={(e) => onToggleCompleted(lec.id, e)}
                      title={isCompleted ? 'Mark incomplete' : 'Mark complete'}
                      aria-label={isCompleted ? 'Mark lecture as incomplete' : 'Mark lecture as completed'}
                    >
                      {#if isCompleted}
                        <Check class="w-3.5 h-3.5 stroke-[2.8]" />
                      {/if}
                    </button>

                    <!-- Title & Duration -->
                    <div class="flex-1 min-w-0">
                      <div class="text-xs leading-snug font-medium truncate mb-1 {isActive ? 'text-white font-semibold' : 'group-hover:text-foreground text-[#d8d5e8]'}">
                        {lec.title}
                      </div>
                      <div class="flex items-center gap-1.5 text-[10px] {isActive ? 'text-[#c0b8f0]' : 'text-muted-foreground'}">
                        <Play class="w-2.5 h-2.5 fill-current {isActive ? 'text-[#c0b8f0]' : ''}" />
                        <span>{formatTime(lec.duration_seconds)}</span>
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
