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
    ArrowRight,
  } from 'lucide-svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    courses: CourseSummary[];
    activeCourseId: string | null;
    onSelectCourse: (courseId: string) => void;
    onImportCourse: () => void;
    onRescanCourse: (courseId: string, event: MouseEvent) => void;
    onDeleteCourse: (courseId: string, event: MouseEvent) => void;
  }

  let {
    courses,
    activeCourseId,
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
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 pb-6 border-b border-border/80">
      <div>
        <div class="flex items-center gap-2.5 mb-2">
          <div class="w-8 h-8 rounded-lg bg-[#a435f0]/15 flex items-center justify-center text-[#a435f0]">
            <GraduationCap class="w-5 h-5" />
          </div>
          <h1 class="text-3xl font-black text-foreground tracking-tight">My learning</h1>
        </div>
        <p class="text-sm text-muted-foreground">
          Manage and watch all your offline courses in one place.
        </p>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-3">
        <Button variant="purple" size="default" class="gap-2 shadow-md hover:shadow-[#a435f0]/20" onclick={onImportCourse}>
          <FolderPlus class="w-4 h-4" />
          <span>Import Course Folder</span>
        </Button>
      </div>
    </div>

    <!-- Quick Stats Overview Bar -->
    {#if courses.length > 0}
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <Card class="p-4 bg-card border-border/70 flex items-center gap-3.5 shadow-xs">
          <div class="w-10 h-10 rounded-lg bg-[#a435f0]/10 flex items-center justify-center text-[#a435f0] shrink-0">
            <BookOpen class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">Courses</div>
            <div class="text-xl font-black text-foreground">{courses.length}</div>
          </div>
        </Card>

        <Card class="p-4 bg-card border-border/70 flex items-center gap-3.5 shadow-xs">
          <div class="w-10 h-10 rounded-lg bg-[#a435f0]/10 flex items-center justify-center text-[#a435f0] shrink-0">
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">Videos</div>
            <div class="text-xl font-black text-foreground">{totalLectures}</div>
          </div>
        </Card>

        <Card class="p-4 bg-card border-border/70 flex items-center gap-3.5 shadow-xs">
          <div class="w-10 h-10 rounded-lg bg-[#a435f0]/10 flex items-center justify-center text-[#a435f0] shrink-0">
            <Clock class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">Total Time</div>
            <div class="text-xl font-black text-foreground">{formatTotalDuration(totalDuration)}</div>
          </div>
        </Card>

        <Card class="p-4 bg-card border-border/70 flex items-center gap-3.5 shadow-xs">
          <div class="w-10 h-10 rounded-lg bg-[#a435f0]/10 flex items-center justify-center text-[#a435f0] shrink-0">
            <CheckCircle2 class="w-5 h-5" />
          </div>
          <div>
            <div class="text-[11px] font-bold text-muted-foreground uppercase tracking-wider">Completed</div>
            <div class="text-xl font-black text-[#a435f0]">{totalCompleted} / {totalLectures}</div>
          </div>
        </Card>
      </div>
    {/if}

    <!-- Search / Filter Bar -->
    {#if courses.length > 0}
      <div class="flex items-center justify-between gap-4">
        <div class="relative flex-1 max-w-md">
          <Search class="w-4 h-4 absolute left-3 text-muted-foreground pointer-events-none" />
          <Input
            type="text"
            placeholder="Search your courses..."
            class="pl-9 h-10 text-sm bg-card border-border/80 focus:bg-background"
            bind:value={searchQuery}
          />
        </div>
        <span class="text-xs font-semibold text-muted-foreground">
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
              class="group overflow-hidden bg-card border-border/80 hover:border-[#a435f0] hover:shadow-xl hover:-translate-y-1.5 transition-all duration-300 flex flex-col h-full shadow-xs cursor-pointer {isActive
                ? 'ring-2 ring-[#a435f0]/50 border-[#a435f0]'
                : ''}"
              onclick={() => onSelectCourse(course.id)}
            >
              <!-- Course Card Header Banner -->
              <div class="h-32 bg-gradient-to-br from-[#4c1d95] via-[#581c87] to-[#1e1b4b] p-5 flex flex-col justify-between relative overflow-hidden border-b border-border/60">
                <div class="absolute -right-6 -bottom-6 w-28 h-28 rounded-full bg-[#a435f0]/30 blur-xl pointer-events-none group-hover:bg-[#a435f0]/40 transition-colors"></div>
                
                <div class="flex items-center justify-between z-10">
                  <span class="bg-[#a435f0] text-white text-[10px] font-black uppercase tracking-wider px-2 py-0.5 rounded shadow-sm">
                    Offline
                  </span>
                  {#if isActive}
                    <span class="bg-emerald-500/30 text-emerald-300 text-[10px] font-extrabold px-2 py-0.5 rounded-full border border-emerald-400/40">
                      Currently Playing
                    </span>
                  {/if}
                </div>

                <div class="z-10 flex items-center justify-between text-zinc-200 text-xs font-semibold">
                  <span>{course.stats.total_sections} sections</span>
                  <span>{course.stats.total_lectures} lectures</span>
                  <span>{formatTotalDuration(course.stats.total_duration_seconds)}</span>
                </div>
              </div>

              <!-- Course Card Body -->
              <div class="p-5 flex-1 flex flex-col justify-between gap-4">
                <div>
                  <h3
                    class="font-bold text-base text-foreground group-hover:text-[#a435f0] transition-colors line-clamp-2 leading-snug mb-2"
                    title={course.title}
                  >
                    {course.title}
                  </h3>
                  <div class="text-xs text-muted-foreground font-mono truncate bg-secondary/40 px-2 py-1 rounded border border-border/40" title={course.root_path}>
                    {course.root_path}
                  </div>
                </div>

                <!-- Progress & Actions -->
                <div class="space-y-4 pt-2">
                  <div class="space-y-1.5">
                    <div class="flex items-center justify-between text-xs font-bold">
                      <span class="text-muted-foreground">{course.stats.completed_lectures}/{course.stats.total_lectures} completed</span>
                      <span class="text-foreground">{pct}%</span>
                    </div>
                    <Progress value={pct} class="h-2" />
                  </div>

                  <div class="flex items-center justify-between pt-2 border-t border-border/50">
                    <Button
                      variant="purple"
                      size="sm"
                      class="gap-1.5 font-bold text-xs shadow-xs"
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
                        title="Rescan folder from disk"
                      >
                        <RefreshCw class="w-3.5 h-3.5 transition-transform duration-300 hover:rotate-180" />
                      </Button>
                      <Button
                        variant="secondary"
                        size="icon"
                        class="w-8 h-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                        onclick={(e) => onDeleteCourse(course.id, e)}
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
      <div class="text-center py-20 bg-card rounded-2xl border border-dashed border-border p-10 flex flex-col items-center justify-center gap-4 max-w-lg mx-auto shadow-sm" in:fade={{ duration: 250 }}>
        <div class="w-20 h-20 rounded-2xl bg-[#a435f0]/10 flex items-center justify-center text-[#a435f0] shadow-inner">
          <BookOpen class="w-10 h-10" />
        </div>
        <h2 class="text-2xl font-black text-foreground">No courses imported yet</h2>
        <p class="text-sm text-muted-foreground text-center">
          Import your local course folders containing video lessons and start watching with an authentic Udemy offline experience.
        </p>
        <Button variant="purple" size="lg" class="mt-3 gap-2 font-bold shadow-md hover:shadow-[#a435f0]/25" onclick={onImportCourse}>
          <FolderPlus class="w-5 h-5" />
          <span>Import Course Folder</span>
        </Button>
      </div>
    {:else}
      <div class="text-center py-16 text-muted-foreground text-sm">
        No courses matching "{searchQuery}".
      </div>
    {/if}
  </div>
</div>
