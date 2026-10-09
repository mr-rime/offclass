<script lang="ts">
  import type { CourseSummary } from '../types';
  import { formatTotalDuration } from '../utils';
  import Button from './ui/Button.svelte';
  import Progress from './ui/Progress.svelte';
  import Input from './ui/Input.svelte';
  import Card from './ui/Card.svelte';
  import {
    FolderPlus,
    RefreshCw,
    Trash2,
    Play,
    Search,
    BookOpen,
    GraduationCap,
    Clock,
    CheckCircle2,
    Layers,
    Loader2,
    X,
  } from 'lucide-svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    courses: CourseSummary[];
    activeCourseId: string | null;
    isImporting?: boolean;
    rescanningCourseId?: string | null;
    onSelectCourse: (courseId: string) => void;
    onImportCourse: () => void;
    onRescanCourse: (courseId: string, event: MouseEvent) => void;
    onDeleteCourse: (courseId: string, event: MouseEvent) => void;
  }

  let {
    courses,
    activeCourseId,
    isImporting = false,
    rescanningCourseId = null,
    onSelectCourse,
    onImportCourse,
    onRescanCourse,
    onDeleteCourse,
  }: Props = $props();

  let searchQuery = $state('');

  let filteredCourses = $derived(
    courses.filter(
      (c) =>
        !searchQuery.trim() ||
        c.title.toLowerCase().includes(searchQuery.toLowerCase().trim())
    )
  );

  let totalLectures = $derived(
    courses.reduce((acc, c) => acc + (c.stats.total_lectures || 0), 0)
  );
  let totalDuration = $derived(
    courses.reduce((acc, c) => acc + (c.stats.total_duration_seconds || 0), 0)
  );
  let totalCompleted = $derived(
    courses.reduce((acc, c) => acc + (c.stats.completed_lectures || 0), 0)
  );
</script>

<div class="flex-1 overflow-y-auto bg-background p-6 md:p-10 select-none">
  <div class="max-w-6xl mx-auto space-y-8">
    <!-- Hero / Page Banner -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 pb-6 border-b border-border">
      <div>
        <div class="flex items-center gap-2.5 mb-1.5">
          <div class="w-8 h-8 rounded-md bg-[#22203d] border border-border/80 flex items-center justify-center overflow-hidden shadow-sm">
            <img src="/logo.png" alt="OffClass" class="w-full h-full object-cover rounded-md" />
          </div>
          <h1 class="text-2xl font-bold text-foreground tracking-tight">My learning</h1>
        </div>
        <p class="text-sm text-muted-foreground">
          Manage and watch all your offline courses in one place.
        </p>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-3">
        <Button
          variant="purple"
          size="default"
          class="gap-2"
          onclick={onImportCourse}
          disabled={isImporting}
        >
          {#if isImporting}
            <Loader2 class="w-4 h-4 animate-spin" />
            <span>Importing...</span>
          {:else}
            <FolderPlus class="w-4 h-4" />
            <span>Import Course Folder</span>
          {/if}
        </Button>
      </div>
    </div>

    <!-- Quick Stats Overview Bar -->
    {#if courses.length > 0}
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <Card class="p-4 bg-card border-border flex items-center gap-3.5">
          <div class="w-10 h-10 rounded-lg bg-accent flex items-center justify-center text-primary shrink-0">
            <BookOpen class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Courses</div>
            <div class="text-xl font-bold text-foreground">{courses.length}</div>
          </div>
        </Card>

        <Card class="p-4 bg-card border-border flex items-center gap-3.5">
          <div class="w-10 h-10 rounded-lg bg-accent flex items-center justify-center text-primary shrink-0">
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Videos</div>
            <div class="text-xl font-bold text-foreground">{totalLectures}</div>
          </div>
        </Card>

        <Card class="p-4 bg-card border-border flex items-center gap-3.5">
          <div class="w-10 h-10 rounded-lg bg-accent flex items-center justify-center text-primary shrink-0">
            <Clock class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Total Time</div>
            <div class="text-xl font-bold text-foreground">{formatTotalDuration(totalDuration)}</div>
          </div>
        </Card>

        <Card class="p-4 bg-card border-border flex items-center gap-3.5">
          <div class="w-10 h-10 rounded-lg bg-accent flex items-center justify-center text-primary shrink-0">
            <CheckCircle2 class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Completed</div>
            <div class="text-xl font-bold text-primary">{totalCompleted} / {totalLectures}</div>
          </div>
        </Card>
      </div>
    {/if}

    <!-- Search / Filter Bar -->
    {#if courses.length > 0}
      <div class="flex items-center justify-between gap-4">
        <div class="relative flex-1 max-w-md flex items-center">
          <Search class="w-4 h-4 absolute left-3.5 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none transition-colors" />
          <Input
            type="text"
            placeholder="Search your courses..."
            class="pl-10 pr-9 h-10 text-sm bg-card border-border focus:border-primary focus:bg-background rounded-lg shadow-xs"
            bind:value={searchQuery}
          />
          {#if searchQuery}
            <button
              class="absolute right-2.5 top-1/2 -translate-y-1/2 w-6 h-6 rounded-md hover:bg-secondary flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
              onclick={() => (searchQuery = '')}
              title="Clear search"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          {/if}
        </div>
        <span class="text-xs font-medium text-muted-foreground">
          Showing {filteredCourses.length} of {courses.length} courses
        </span>
      </div>
    {/if}

    <!-- Courses Grid -->
    {#if filteredCourses.length > 0}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each filteredCourses as course, idx (course.id)}
          {@const pct = Math.round(course.stats.progress_percent || 0)}
          {@const isActive = activeCourseId === course.id}

          <div in:fly={{ y: 20, duration: 250 + idx * 40, easing: cubicOut }}>
            <Card
              class="group overflow-hidden bg-card border-border hover:border-primary transition-all duration-200 flex flex-col h-full cursor-pointer {isActive
                ? 'ring-2 ring-primary/60 border-primary'
                : ''}"
              onclick={() => onSelectCourse(course.id)}
            >
              <!-- Course Card Header Banner (Solid Clean Surface) -->
              <div class="h-28 bg-[#22203d] p-4 flex flex-col justify-between relative border-b border-border">
                <div class="flex items-center justify-between z-10">
                  <span class="bg-primary text-primary-foreground text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded">
                    Offline
                  </span>
                  {#if isActive}
                    <span class="bg-[#362857] text-[#c0b8f0] text-[10px] font-bold px-2 py-0.5 rounded border border-primary/40">
                      Currently Playing
                    </span>
                  {/if}
                </div>

                <div class="z-10 flex items-center justify-between text-[#827f9e] text-xs font-medium">
                  <span>{course.stats.total_sections} sections</span>
                  <span>{course.stats.total_lectures} lectures</span>
                  <span>{formatTotalDuration(course.stats.total_duration_seconds)}</span>
                </div>
              </div>

              <!-- Course Card Body -->
              <div class="p-5 flex-1 flex flex-col justify-between gap-4">
                <div>
                  <h3
                    class="font-semibold text-base text-foreground group-hover:text-primary transition-colors line-clamp-2 leading-snug mb-2"
                    title={course.title}
                  >
                    {course.title}
                  </h3>
                  <div class="text-xs text-muted-foreground font-mono truncate bg-secondary/50 px-2 py-1 rounded border border-border/60" title={course.root_path}>
                    {course.root_path}
                  </div>
                </div>

                <!-- Progress & Actions -->
                <div class="space-y-4 pt-2">
                  <div class="space-y-1.5">
                    <div class="flex items-center justify-between text-xs font-medium">
                      <span class="text-muted-foreground">{course.stats.completed_lectures}/{course.stats.total_lectures} completed</span>
                      <span class="text-foreground font-semibold">{pct}%</span>
                    </div>
                    <Progress value={pct} class="h-2" />
                  </div>

                  <div class="flex items-center justify-between pt-3 border-t border-border/80">
                    <Button
                      variant="purple"
                      size="sm"
                      class="gap-1.5 font-semibold text-xs"
                      onclick={(e) => {
                        e.stopPropagation();
                        onSelectCourse(course.id);
                      }}
                    >
                      <Play class="w-3.5 h-3.5 fill-current" />
                      <span>{pct > 0 ? 'Resume' : 'Start'}</span>
                    </Button>

                    <div class="flex items-center gap-1">
                      <Button
                        variant="secondary"
                        size="icon"
                        class="w-8 h-8 text-muted-foreground hover:text-foreground"
                        onclick={(e) => onRescanCourse(course.id, e)}
                        disabled={rescanningCourseId === course.id || isImporting}
                        title="Rescan folder from disk"
                      >
                        {#if rescanningCourseId === course.id}
                          <Loader2 class="w-3.5 h-3.5 animate-spin text-primary" />
                        {:else}
                          <RefreshCw class="w-3.5 h-3.5 transition-transform duration-300 hover:rotate-180" />
                        {/if}
                      </Button>
                      <Button
                        variant="secondary"
                        size="icon"
                        class="w-8 h-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                        onclick={(e) => onDeleteCourse(course.id, e)}
                        disabled={rescanningCourseId === course.id || isImporting}
                        title="Remove from library"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            </Card>
          </div>
        {/each}
      </div>
    {:else if courses.length === 0}
      <!-- Empty State -->
      <div class="text-center py-20 bg-card rounded-xl border border-dashed border-border p-10 flex flex-col items-center justify-center gap-4 max-w-lg mx-auto" in:fade={{ duration: 250 }}>
        <div class="w-16 h-16 rounded-xl bg-[#22203d] border border-border/80 flex items-center justify-center overflow-hidden shadow-md">
          <img src="/logo.png" alt="OffClass" class="w-full h-full object-cover rounded-xl" />
        </div>
        <h2 class="text-xl font-bold text-foreground">No courses imported yet</h2>
        <p class="text-sm text-muted-foreground text-center">
          Import your local course folders containing video lessons and start watching offline.
        </p>
        <Button
          variant="purple"
          size="default"
          class="mt-2 gap-2 font-semibold"
          onclick={onImportCourse}
          disabled={isImporting}
        >
          {#if isImporting}
            <Loader2 class="w-4 h-4 animate-spin" />
            <span>Importing Course...</span>
          {:else}
            <FolderPlus class="w-4 h-4" />
            <span>Import Course Folder</span>
          {/if}
        </Button>
      </div>
    {:else}
      <div class="text-center py-16 text-muted-foreground text-sm">
        No courses matching "{searchQuery}".
      </div>
    {/if}
  </div>
</div>
