<script lang="ts">
  import { onMount } from 'svelte';
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
        applyTheme(userSettings.theme);
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  function applyTheme(theme: 'dark' | 'light') {
    document.documentElement.setAttribute('data-theme', theme);
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }

  async function handleToggleTheme() {
    const nextTheme = userSettings.theme === 'dark' ? 'light' : 'dark';
    userSettings.theme = nextTheme;
    applyTheme(nextTheme);
    try {
      await api.updateSettings(userSettings);
    } catch (e) {}
  }

  async function fetchCoursesAndOpenLast() {
    try {
      const list = await api.listCourses();
      courses = list;
      if (list && list.length > 0) {
        const targetId = userSettings.active_course_id || list[0].id;
        await loadCourse(targetId);
      } else {
        activeView = 'library';
      }
    } catch (e) {
      console.error('Failed to load course list:', e);
    }
  }

  async function loadCourse(courseId: string) {
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
          selectLecture(targetLecture, targetSection);
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

  function selectLecture(lecture: Lecture, section: Section) {
    activeLecture = lecture;
    activeSection = section;
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
    if (currentCourse && activeLecture) {
      activeLecture.duration_seconds = durSec;
      api.updateDuration(currentCourse.id, activeLecture.id, durSec).catch(() => {});
    }
  }

  function handleVideoEnded() {
    if (activeLecture && currentProgress) {
      if (!currentProgress.completed_lecture_ids.includes(activeLecture.id)) {
        handleToggleCompleted(activeLecture.id, new MouseEvent('click'));
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
          selectLecture(l, s);
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
            selectLecture(l, s);
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
        const detail = await api.scanCourse(folder);
        if (detail) {
          courses = await api.listCourses();
          await loadCourse(detail.course.id);
        }
      }
    } catch (e) {
      alert(`Could not import course folder: ${e}`);
    }
  }

  async function handleRescanCourse(courseId: string, event: MouseEvent) {
    event.stopPropagation();
    try {
      const detail = await api.rescanCourse(courseId);
      if (detail) {
        courses = await api.listCourses();
        if (currentCourse?.id === courseId) {
          await loadCourse(courseId);
        }
      }
    } catch (e) {
      alert(`Rescan failed: ${e}`);
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
    theme={userSettings.theme}
    {sidebarOpen}
    onToggleTheme={handleToggleTheme}
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
      onSelectCourse={loadCourse}
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
          savedPosition={(activeLecture && currentProgress?.playback_positions?.[activeLecture.id]) || 0}
          onPositionUpdated={handlePositionUpdated}
          onDurationUpdated={handleDurationUpdated}
          onVideoEnded={handleVideoEnded}
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
        onSelectLecture={selectLecture}
        onToggleCompleted={handleToggleCompleted}
        onCloseSidebar={() => (sidebarOpen = false)}
      />
    </div>
  {/if}
</div>
