<script lang="ts">
  import type { Course, CourseProgress, CourseStats, Lecture, Section, Note } from '../types';
  import { formatTime, formatTotalDuration } from '../utils';
  import Button from './ui/Button.svelte';
  import Textarea from './ui/Textarea.svelte';
  import Card from './ui/Card.svelte';
  import {
    Clock,
    Folder,
    Trash2,
    Play,
  } from 'lucide-svelte';
  import { fly, slide, scale, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    course: Course | null;
    stats: CourseStats | null;
    progress: CourseProgress | null;
    activeLecture: Lecture | null;
    activeSection: Section | null;
    currentTime: number;
    onSaveNote: (text: string, timestamp: number) => void;
    onDeleteNote: (noteId: string) => void;
    onSeekToNote: (lectureId: string, timestamp: number) => void;
  }

  let {
    course,
    stats,
    progress,
    activeLecture,
    activeSection,
    currentTime,
    onSaveNote,
    onDeleteNote,
    onSeekToNote,
  }: Props = $props();

  let activeTab = $state<'overview' | 'notes' | 'shortcuts'>('overview');
  let noteText = $state('');

  let completedCount = $derived(progress?.completed_lecture_ids?.length ?? 0);
  let totalCount = $derived(stats?.total_lectures ?? 0);
  let progressPct = $derived(
    totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0
  );

  function handleSaveNote() {
    if (!noteText.trim()) return;
    onSaveNote(noteText.trim(), currentTime);
    noteText = '';
  }
</script>

<div class="max-w-5xl mx-auto w-full p-6 select-none">
  <!-- Lecture Header Info -->
  {#if activeLecture}
    <div class="mb-6">
      {#key activeLecture.id}
        <div in:fly={{ y: -6, duration: 250, easing: cubicOut }}>
          <div class="text-xs font-bold text-[#a435f0] tracking-wider uppercase mb-1">
            {activeSection?.title ?? 'Section'}
          </div>
          <h1 class="text-2xl font-bold text-foreground">
            {activeLecture.order}. {activeLecture.title}
          </h1>
        </div>
      {/key}
    </div>
  {/if}

  <!-- Tab Navigation -->
  <div class="flex gap-6 border-b border-border mb-6">
    <button
      class="pb-3 text-sm font-semibold transition-all duration-200 relative cursor-pointer {activeTab ===
      'overview'
        ? 'text-foreground font-bold'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (activeTab = 'overview')}
    >
      Overview & Stats
      {#if activeTab === 'overview'}
        <div
          class="absolute bottom-0 left-0 right-0 h-0.5 bg-[#a435f0] rounded-t"
          in:scale={{ duration: 200, easing: cubicOut }}
        ></div>
      {/if}
    </button>

    <button
      class="pb-3 text-sm font-semibold transition-all duration-200 relative cursor-pointer {activeTab ===
      'notes'
        ? 'text-foreground font-bold'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (activeTab = 'notes')}
    >
      Notes & Bookmarks ({progress?.notes?.length ?? 0})
      {#if activeTab === 'notes'}
        <div
          class="absolute bottom-0 left-0 right-0 h-0.5 bg-[#a435f0] rounded-t"
          in:scale={{ duration: 200, easing: cubicOut }}
        ></div>
      {/if}
    </button>

    <button
      class="pb-3 text-sm font-semibold transition-all duration-200 relative cursor-pointer {activeTab ===
      'shortcuts'
        ? 'text-foreground font-bold'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (activeTab = 'shortcuts')}
    >
      Keyboard Shortcuts
      {#if activeTab === 'shortcuts'}
        <div
          class="absolute bottom-0 left-0 right-0 h-0.5 bg-[#a435f0] rounded-t"
          in:scale={{ duration: 200, easing: cubicOut }}
        ></div>
      {/if}
    </button>
  </div>

  <!-- Tab Content with smooth animated transitions -->
  {#if activeTab === 'overview'}
    <div in:fly={{ y: 10, duration: 250, easing: cubicOut }}>
      <!-- Stats Grid -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <Card class="p-4 bg-card border-border/70 flex flex-col gap-1 shadow-xs hover:-translate-y-1 hover:shadow-md hover:border-[#a435f0]/50 transition-all duration-300">
          <span class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">
            Modules / Sections
          </span>
          <span class="text-2xl font-extrabold text-foreground">
            {stats?.total_sections ?? 0}
          </span>
        </Card>

        <Card class="p-4 bg-card border-border/70 flex flex-col gap-1 shadow-xs hover:-translate-y-1 hover:shadow-md hover:border-[#a435f0]/50 transition-all duration-300">
          <span class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">
            Total Videos
          </span>
          <span class="text-2xl font-extrabold text-foreground">
            {stats?.total_lectures ?? 0}
          </span>
        </Card>

        <Card class="p-4 bg-card border-border/70 flex flex-col gap-1 shadow-xs hover:-translate-y-1 hover:shadow-md hover:border-[#a435f0]/50 transition-all duration-300">
          <span class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">
            Total Duration
          </span>
          <span class="text-2xl font-extrabold text-foreground">
            {formatTotalDuration(stats?.total_duration_seconds ?? 0)}
          </span>
        </Card>

        <Card class="p-4 bg-card border-border/70 flex flex-col gap-1 shadow-xs hover:-translate-y-1 hover:shadow-md hover:border-[#a435f0]/50 transition-all duration-300">
          <span class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">
            Completed
          </span>
          <span class="text-2xl font-extrabold text-[#a435f0]">
            {completedCount} / {totalCount} ({progressPct}%)
          </span>
        </Card>
      </div>

      <!-- Course Location -->
      <Card class="p-5 bg-card border-border/70 shadow-xs hover:border-[#a435f0]/40 transition-all duration-300">
        <div class="flex items-center gap-2 font-bold text-sm text-foreground mb-2">
          <Folder class="w-4 h-4 text-[#a435f0]" />
          <span>Course Location on Disk</span>
        </div>
        <div class="text-xs text-muted-foreground font-mono bg-secondary/50 p-2.5 rounded border border-border/50 break-all select-text">
          {course?.root_path ?? 'No directory imported'}
        </div>
      </Card>
    </div>
  {:else if activeTab === 'notes'}
    <div in:fly={{ y: 10, duration: 250, easing: cubicOut }}>
      <!-- Create Note Box -->
      <Card class="p-4 bg-card border-border/70 mb-6 shadow-xs hover:border-[#a435f0]/50 transition-all duration-300">
        <Textarea
          placeholder="Type your notes at current timestamp..."
          bind:value={noteText}
          class="mb-3 text-xs focus:ring-[#a435f0]"
          rows={3}
        />
        <div class="flex items-center justify-between">
          <div class="inline-flex items-center gap-1.5 bg-[#a435f0]/15 text-[#a435f0] dark:text-[#c0c4fc] px-2.5 py-1 rounded text-xs font-bold transition-transform hover:scale-105">
            <Clock class="w-3.5 h-3.5" />
            <span>{formatTime(currentTime)}</span>
          </div>
          <Button variant="purple" size="sm" onclick={handleSaveNote}>
            Save Note
          </Button>
        </div>
      </Card>

      <!-- Notes List with animated entry/exit -->
      <div class="space-y-3">
        {#if progress && progress.notes && progress.notes.length > 0}
          {#each progress.notes as note (note.id)}
            <div
              in:fly={{ y: 12, duration: 250, easing: cubicOut }}
              out:slide={{ duration: 150 }}
            >
              <Card class="p-4 bg-card border-border/70 flex items-start gap-4 shadow-xs hover:border-[#a435f0]/60 hover:-translate-y-0.5 transition-all duration-200">
                <button
                  class="inline-flex items-center gap-1 bg-[#a435f0]/15 hover:bg-[#a435f0]/30 active:scale-95 text-[#a435f0] dark:text-[#c0c4fc] px-2.5 py-1 rounded text-xs font-bold cursor-pointer transition-all duration-200"
                  onclick={() => onSeekToNote(note.lecture_id, note.timestamp_seconds)}
                  title="Jump to video timestamp"
                >
                  <Play class="w-3 h-3 fill-current" />
                  <span>{formatTime(note.timestamp_seconds)}</span>
                </button>
                <div class="flex-1 min-w-0">
                  <div class="text-xs font-semibold text-muted-foreground mb-1">
                    {note.lecture_title}
                  </div>
                  <p class="text-xs text-foreground whitespace-pre-wrap leading-relaxed select-text">
                    {note.text}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  class="w-7 h-7 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                  onclick={() => onDeleteNote(note.id)}
                  title="Delete note"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </Button>
              </Card>
            </div>
          {/each}
        {:else}
          <div class="text-center py-12 text-xs text-muted-foreground" in:fade={{ duration: 200 }}>
            No notes yet. Type a note above to bookmark important moments!
          </div>
        {/if}
      </div>
    </div>
  {:else if activeTab === 'shortcuts'}
    <div in:fly={{ y: 10, duration: 250, easing: cubicOut }} class="grid grid-cols-2 md:grid-cols-3 gap-3">
      <Card class="p-3.5 bg-card border-border/70 flex items-center justify-between hover:border-[#a435f0]/40 transition-all duration-200 hover:-translate-y-0.5">
        <span class="text-xs font-semibold text-muted-foreground">Play / Pause</span>
        <kbd class="px-2 py-1 bg-secondary text-foreground rounded text-xs font-mono font-bold border border-border shadow-xs">Space / K</kbd>
      </Card>
      <Card class="p-3.5 bg-card border-border/70 flex items-center justify-between hover:border-[#a435f0]/40 transition-all duration-200 hover:-translate-y-0.5">
        <span class="text-xs font-semibold text-muted-foreground">Seek ±5s</span>
        <kbd class="px-2 py-1 bg-secondary text-foreground rounded text-xs font-mono font-bold border border-border shadow-xs">← / →</kbd>
      </Card>
      <Card class="p-3.5 bg-card border-border/70 flex items-center justify-between hover:border-[#a435f0]/40 transition-all duration-200 hover:-translate-y-0.5">
        <span class="text-xs font-semibold text-muted-foreground">Volume ±10%</span>
        <kbd class="px-2 py-1 bg-secondary text-foreground rounded text-xs font-mono font-bold border border-border shadow-xs">↑ / ↓</kbd>
      </Card>
      <Card class="p-3.5 bg-card border-border/70 flex items-center justify-between hover:border-[#a435f0]/40 transition-all duration-200 hover:-translate-y-0.5">
        <span class="text-xs font-semibold text-muted-foreground">Mute / Unmute</span>
        <kbd class="px-2 py-1 bg-secondary text-foreground rounded text-xs font-mono font-bold border border-border shadow-xs">M</kbd>
      </Card>
      <Card class="p-3.5 bg-card border-border/70 flex items-center justify-between hover:border-[#a435f0]/40 transition-all duration-200 hover:-translate-y-0.5">
        <span class="text-xs font-semibold text-muted-foreground">Fullscreen</span>
        <kbd class="px-2 py-1 bg-secondary text-foreground rounded text-xs font-mono font-bold border border-border shadow-xs">F</kbd>
      </Card>
      <Card class="p-3.5 bg-card border-border/70 flex items-center justify-between hover:border-[#a435f0]/40 transition-all duration-200 hover:-translate-y-0.5">
        <span class="text-xs font-semibold text-muted-foreground">Next / Prev Video</span>
        <kbd class="px-2 py-1 bg-secondary text-foreground rounded text-xs font-mono font-bold border border-border shadow-xs">N / P</kbd>
      </Card>
    </div>
  {/if}
</div>
