<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import Header from './lib/components/Header.svelte';
  import VideoPlayer from './lib/components/VideoPlayer.svelte';
  import CourseContentSidebar from './lib/components/CourseContentSidebar.svelte';
  import CourseTabs from './lib/components/CourseTabs.svelte';
  import CoursesPage from './lib/components/CoursesPage.svelte';
  import { api } from './lib/tauri';
  import type { Course, CourseProgress, CourseStats, CourseSummary, Lecture, Section, UserSettings } from './lib/types';

  // App Navigation & State
  let activeView = $state<'player' | 'library'>('library');
  let courses: CourseSummary[] = $state([]);
  let currentCourse: Course | null = $state(null);
  let currentProgress: CourseProgress | null = $state(null);
  let currentStats: CourseStats | null = $state(null);
  let activeLecture: Lecture | null = $state(null);
  let activeSection: Section | null = $state(null);
  let streamingPort = $state(0);
  let userSettings: UserSettings = $state({
    theme: 'dark',
    auto_play_next: true,
    playback_speed: 1.0,
    volume: 1.0,
    active_course_id: null,
  });

  let sidebarOpen = $state(true);
  let videoPlayerRef: VideoPlayer | null = $state(null);
  let currentVideoTime = $state(0);
  let shouldAutoPlay = $state(false);
  let isImporting = $state(false);
  let rescanningCourseId = $state<string | null>(null);

  onMount(async () => {
    try {
      streamingPort = await api.getStreamingPort();
    } catch (e) {
      console.warn('Could not get streaming port:', e);
    }

    await loadSettings();
    await fetchCoursesAndOpenLast();
  });

  async function loadSettings() {
    try {
      const settings = await api.getSettings();
      if (settings) {
        userSettings = settings;
        applyTheme();
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  function applyTheme() {
    document.documentElement.setAttribute('data-theme', 'dark');
    document.documentElement.classList.add('dark');
  }

  async function fetchCoursesAndOpenLast() {
    try {
      const list = await api.listCourses();
      courses = list;
      if (list && list.length > 0) {
        const targetId = userSettings.active_course_id || list[0].id;
        await loadCourse(targetId, false);
      } else {
        activeView = 'library';
      }
    } catch (e) {
      console.error('Failed to load course list:', e);
    }
  }

  async function loadCourse(courseId: string, autoPlay: boolean = false) {
    try {
      const detail = await api.getCourse(courseId);
      if (detail) {
        currentCourse = detail.course;
        currentProgress = detail.progress;
        currentStats = detail.stats;

        userSettings.active_course_id = currentCourse.id;
        api.updateSettings(userSettings).catch(() => {});

        // Select last played lecture or first lecture
        let targetLecture: Lecture | null = null;
        let targetSection: Section | null = null;

        if (currentProgress.last_played_lecture_id) {
          for (const s of currentCourse.sections) {
            for (const l of s.lectures) {
              if (l.id === currentProgress.last_played_lecture_id) {
                targetLecture = l;
                targetSection = s;
                break;
              }
            }
            if (targetLecture) break;
          }
        }

        if (!targetLecture && currentCourse.sections.length > 0 && currentCourse.sections[0].lectures.length > 0) {
          targetSection = currentCourse.sections[0];
          targetLecture = currentCourse.sections[0].lectures[0];
        }

        if (targetLecture && targetSection) {
          selectLecture(targetLecture, targetSection, autoPlay);
        }

        // Navigate to player
        activeView = 'player';
        // Refresh courses list in background
        api.listCourses().then((c) => (courses = c)).catch(() => {});
      }
    } catch (e) {
      console.error('Failed to load course:', e);
    }
  }

  function selectLecture(lecture: Lecture, section: Section, autoPlay: boolean = false) {
    if (currentCourse && activeLecture && videoPlayerRef) {
      const cur = videoPlayerRef.getCurrentTime();
      if (cur > 0) {
        handlePositionUpdated(cur);
      }
    }

    shouldAutoPlay = autoPlay;
    activeLecture = lecture;
    activeSection = section;

    if (currentCourse) {
      if (!currentProgress) {
        currentProgress = {
          course_id: currentCourse.id,
          completed_lecture_ids: [],
          last_played_lecture_id: lecture.id,
          playback_positions: {},
          notes: [],
        };
      }
      currentProgress.last_played_lecture_id = lecture.id;
      const existingPos = currentProgress.playback_positions[lecture.id] || 0;
      api.updatePosition(currentCourse.id, lecture.id, existingPos).catch(() => {});
    }
  }

  async function handleToggleCompleted(lectureId: string, event: MouseEvent) {
    event.stopPropagation();
    if (!currentCourse || !currentProgress) return;

    const isCompleted = currentProgress.completed_lecture_ids?.includes(lectureId) ?? false;
    const nextStatus = !isCompleted;

    // Optimistic update
    if (nextStatus) {
      currentProgress.completed_lecture_ids = [...(currentProgress.completed_lecture_ids || []), lectureId];
    } else {
      currentProgress.completed_lecture_ids = currentProgress.completed_lecture_ids.filter((id) => id !== lectureId);
    }

    try {
      const detail = await api.updateProgress(currentCourse.id, lectureId, nextStatus);
      if (detail) {
        currentProgress = detail.progress;
        currentStats = detail.stats;
        // Also update summary list
        courses = await api.listCourses();
      }
    } catch (e) {
      console.error('Failed to update progress:', e);
    }
  }

  function handlePositionUpdated(posSec: number) {
    currentVideoTime = posSec;
    if (currentCourse && activeLecture) {
      if (!currentProgress) {
        currentProgress = {
          course_id: currentCourse.id,
          completed_lecture_ids: [],
          last_played_lecture_id: activeLecture.id,
          playback_positions: {},
          notes: [],
        };
      }
      currentProgress.playback_positions[activeLecture.id] = posSec;
      currentProgress.last_played_lecture_id = activeLecture.id;

      api.updatePosition(currentCourse.id, activeLecture.id, posSec).catch(() => {});
    }
  }

  function handleDurationUpdated(durSec: number) {
    if (currentCourse && activeLecture && durSec > 0) {
      activeLecture.duration_seconds = durSec;
      if (activeSection) {
        activeSection.duration_seconds = activeSection.lectures.reduce(
          (acc, l) => acc + (l.duration_seconds || 0),
          0
        );
      }
      api.updateDuration(currentCourse.id, activeLecture.id, durSec).catch(() => {});
    }
  }

  function handleVideoEnded() {
    if (activeLecture && currentProgress && currentCourse) {
      if (!currentProgress.completed_lecture_ids.includes(activeLecture.id)) {
        handleToggleCompleted(activeLecture.id, new MouseEvent('click'));
      }
      if (activeLecture.duration_seconds > 0) {
        handlePositionUpdated(activeLecture.duration_seconds);
      }
      if (userSettings.auto_play_next) {
        playNextLecture();
      }
    }
  }

  function playNextLecture() {
    if (!currentCourse || !activeLecture) return;
    let found = false;
    for (const s of currentCourse.sections) {
      for (const l of s.lectures) {
        if (found) {
          selectLecture(l, s, true);
          return;
        }
        if (l.id === activeLecture.id) {
          found = true;
        }
      }
    }
  }

  async function handleSaveNote(text: string, timestamp: number) {
    if (!currentCourse || !activeLecture) return;
    try {
      const notes = await api.addNote(
        currentCourse.id,
        activeLecture.id,
        activeLecture.title,
        timestamp,
        text
      );
      if (currentProgress) {
        currentProgress.notes = notes;
      }
    } catch (e) {
      console.error('Failed to save note:', e);
    }
  }

  async function handleDeleteNote(noteId: string) {
    if (!currentCourse) return;
    try {
      const notes = await api.deleteNote(currentCourse.id, noteId);
      if (currentProgress) {
        currentProgress.notes = notes;
      }
    } catch (e) {
      console.error('Failed to delete note:', e);
    }
  }

  function handleSeekToNote(lectureId: string, timestamp: number) {
    if (activeLecture?.id === lectureId) {
      videoPlayerRef?.seekTo(timestamp);
    } else if (currentCourse) {
      for (const s of currentCourse.sections) {
        for (const l of s.lectures) {
          if (l.id === lectureId) {
            selectLecture(l, s, true);
            setTimeout(() => {
              videoPlayerRef?.seekTo(timestamp);
            }, 300);
            return;
          }
        }
      }
    }
  }

  async function handleImportCourse() {
    try {
      const folder = await api.pickFolder();
      if (folder) {
        isImporting = true;
        const detail = await api.scanCourse(folder);
        if (detail) {
          courses = await api.listCourses();
          await loadCourse(detail.course.id);
        }
      }
    } catch (e) {
      alert(`Could not import course folder: ${e}`);
    } finally {
      isImporting = false;
    }
  }

  async function handleRescanCourse(courseId: string, event: MouseEvent) {
    event.stopPropagation();
    try {
      rescanningCourseId = courseId;
      const detail = await api.rescanCourse(courseId);
      if (detail) {
        courses = await api.listCourses();
        if (currentCourse?.id === courseId) {
          await loadCourse(courseId);
        }
      }
    } catch (e) {
      alert(`Rescan failed: ${e}`);
    } finally {
      rescanningCourseId = null;
    }
  }

  async function handleDeleteCourse(courseId: string, event: MouseEvent) {
    event.stopPropagation();
    if (!confirm('Are you sure you want to remove this course from your library?')) return;
    try {
      await api.deleteCourse(courseId);
      courses = await api.listCourses();
      if (currentCourse?.id === courseId) {
        if (courses.length > 0) {
          await loadCourse(courses[0].id);
        } else {
          currentCourse = null;
          currentProgress = null;
          currentStats = null;
          activeLecture = null;
          activeSection = null;
          activeView = 'library';
        }
      }
    } catch (e) {}
  }
</script>

<div class="h-screen w-screen flex flex-col bg-background text-foreground overflow-hidden font-sans">
  <!-- Top Navigation Header -->
  <Header
    {activeView}
    {currentCourse}
    stats={currentStats}
    progress={currentProgress}
    {sidebarOpen}
    {isImporting}
    onToggleSidebar={() => (sidebarOpen = !sidebarOpen)}
    onNavigateLibrary={() => (activeView = 'library')}
    onNavigatePlayer={() => (activeView = 'player')}
    onImportCourse={handleImportCourse}
  />

  <!-- Main View Area -->
  {#if activeView === 'library'}
    <!-- Dedicated "My learning" Courses Page -->
    <CoursesPage
      {courses}
      activeCourseId={currentCourse?.id ?? null}
      {isImporting}
      {rescanningCourseId}
      onSelectCourse={(id) => loadCourse(id, true)}
      onImportCourse={handleImportCourse}
      onRescanCourse={handleRescanCourse}
      onDeleteCourse={handleDeleteCourse}
    />
  {:else}
    <!-- Course Player View -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left / Center Player & Details Scrollable Area -->
      <main class="flex-1 flex flex-col overflow-y-auto bg-background">
        <VideoPlayer
          bind:this={videoPlayerRef}
          {currentCourse}
          {activeLecture}
          {activeSection}
          {streamingPort}
          {isImporting}
          autoPlay={shouldAutoPlay}
          savedPosition={(activeLecture && currentProgress?.playback_positions?.[activeLecture.id]) || 0}
          notes={currentProgress?.notes?.filter((n) => n.lecture_id === activeLecture?.id) ?? []}
          onPositionUpdated={handlePositionUpdated}
          onDurationUpdated={handleDurationUpdated}
          onVideoEnded={handleVideoEnded}
          onNextLecture={playNextLecture}
          onImportCourse={handleImportCourse}
        />

        <!-- Tabs & Information Below Video -->
        {#if currentCourse && activeLecture}
          <CourseTabs
            course={currentCourse}
            stats={currentStats}
            progress={currentProgress}
            {activeLecture}
            {activeSection}
            currentTime={currentVideoTime}
            onSaveNote={handleSaveNote}
            onDeleteNote={handleDeleteNote}
            onSeekToNote={handleSeekToNote}
          />
        {/if}
      </main>

      <!-- Right Sidebar Drawer -->
      <CourseContentSidebar
        course={currentCourse}
        stats={currentStats}
        progress={currentProgress}
        {activeLecture}
        isOpen={sidebarOpen}
        onSelectLecture={(lec, sec) => selectLecture(lec, sec, true)}
        onToggleCompleted={handleToggleCompleted}
        onCloseSidebar={() => (sidebarOpen = false)}
      />
    </div>
  {/if}

  <!-- Sleek Course Importing Backdrop Modal -->
  {#if isImporting}
    <div
      class="fixed inset-0 z-50 bg-black/75 backdrop-blur-md flex items-center justify-center p-4 select-none"
      transition:fade={{ duration: 180 }}
    >
      <div
        class="bg-[#161528] border border-primary/40 shadow-2xl rounded-2xl p-7 max-w-sm w-full flex flex-col items-center text-center space-y-4 relative overflow-hidden"
        in:scale={{ start: 0.94, duration: 220, easing: cubicOut }}
      >
        <!-- Glowing top line -->
        <div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-transparent via-primary to-transparent animate-pulse"></div>

        <!-- Animated Logo & Spinner Ring -->
        <div class="relative flex items-center justify-center my-1">
          <div class="w-16 h-16 rounded-2xl bg-[#22203d] border border-border/80 flex items-center justify-center overflow-hidden shadow-lg p-2.5">
            <img src="/logo.png" alt="OffClass" class="w-full h-full object-cover rounded-xl" />
          </div>
          <div class="absolute -inset-2.5 rounded-2xl border-2 border-primary/30 border-t-primary animate-spin"></div>
        </div>

        <!-- Text details -->
        <div class="space-y-1">
          <h3 class="text-base font-bold text-foreground">Importing Course</h3>
          <p class="text-xs text-muted-foreground leading-relaxed">
            Scanning directory and indexing videos & subtitles with ffprobe...
          </p>
        </div>

        <!-- Animated Progress Bar -->
        <div class="w-full bg-[#22203d] h-1.5 rounded-full overflow-hidden">
          <div class="h-full bg-primary rounded-full animate-indeterminate"></div>
        </div>
      </div>
    </div>
  {/if}
</div>
