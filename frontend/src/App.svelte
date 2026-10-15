<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import Header from './lib/components/Header.svelte';
  import VideoPlayer from './lib/components/VideoPlayer.svelte';
  import HtmlViewer from './lib/components/HtmlViewer.svelte';
  import PdfViewer from './lib/components/PdfViewer.svelte';
  import CourseContentSidebar from './lib/components/CourseContentSidebar.svelte';
  import CourseTabs from './lib/components/CourseTabs.svelte';
  import TabBar from './lib/components/TabBar.svelte';
  import CoursesPage from './lib/components/CoursesPage.svelte';
  import { api, listenToEvent } from './lib/tauri';
  import type { Course, CourseProgress, CourseStats, CourseSummary, CourseTab, Lecture, Section, UserSettings, ScanProgress } from './lib/types';

  // App Navigation & State
  let activeView = $state<'player' | 'library'>('library');
  let courses: CourseSummary[] = $state([]);
  let currentCourse: Course | null = $state(null);
  let currentProgress: CourseProgress | null = $state(null);
  let currentStats: CourseStats | null = $state(null);
  let activeVideoLecture: Lecture | null = $state(null);
  let activeVideoSection: Section | null = $state(null);
  let activeLecture: Lecture | null = $state(null);
  let activeSection: Section | null = $state(null);
  let openTabs: CourseTab[] = $state([]);
  let activeTabId = $state<string>('video');
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
  let scanProgress = $state<ScanProgress | null>(null);

  let activeTab = $derived(openTabs.find((t) => t.id === activeTabId) ?? openTabs[0] ?? null);
  let currentDisplayLecture = $derived(activeTab?.lecture ?? activeVideoLecture ?? activeLecture);
  let currentDisplaySection = $derived(activeTab?.section ?? activeVideoSection ?? activeSection);

  function isDocumentLecture(lec: Lecture | null | undefined): boolean {
    if (!lec) return false;
    return (
      lec.item_type === 'html' ||
      lec.item_type === 'pdf' ||
      lec.file_name?.toLowerCase().endsWith('.html') ||
      lec.file_name?.toLowerCase().endsWith('.htm') ||
      lec.file_name?.toLowerCase().endsWith('.pdf') ||
      lec.relative_path?.toLowerCase().endsWith('.html') ||
      lec.relative_path?.toLowerCase().endsWith('.htm') ||
      lec.relative_path?.toLowerCase().endsWith('.pdf') ||
      lec.absolute_path?.toLowerCase().endsWith('.html') ||
      lec.absolute_path?.toLowerCase().endsWith('.htm') ||
      lec.absolute_path?.toLowerCase().endsWith('.pdf')
    );
  }

  function getLectureType(lec: Lecture | null | undefined): 'video' | 'html' | 'pdf' {
    if (!lec) return 'video';
    if (
      lec.item_type === 'pdf' ||
      lec.file_name?.toLowerCase().endsWith('.pdf') ||
      lec.relative_path?.toLowerCase().endsWith('.pdf') ||
      lec.absolute_path?.toLowerCase().endsWith('.pdf')
    ) {
      return 'pdf';
    }
    if (
      lec.item_type === 'html' ||
      lec.file_name?.toLowerCase().endsWith('.html') ||
      lec.file_name?.toLowerCase().endsWith('.htm') ||
      lec.relative_path?.toLowerCase().endsWith('.html') ||
      lec.relative_path?.toLowerCase().endsWith('.htm') ||
      lec.absolute_path?.toLowerCase().endsWith('.html') ||
      lec.absolute_path?.toLowerCase().endsWith('.htm')
    ) {
      return 'html';
    }
    return 'video';
  }

  onMount(async () => {
    try {
      streamingPort = await api.getStreamingPort();
    } catch (e) {
      console.warn('Could not get streaming port:', e);
    }

    listenToEvent<ScanProgress>('scan-progress', (p) => {
      scanProgress = p;
    }).catch(() => {});

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

        // Reset tabs for new course
        openTabs = [];

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

        // Find primary video lecture in course
        let firstVideoLecture: Lecture | null = null;
        let firstVideoSection: Section | null = null;
        for (const s of currentCourse.sections) {
          for (const l of s.lectures) {
            if (!isDocumentLecture(l)) {
              firstVideoLecture = l;
              firstVideoSection = s;
              break;
            }
          }
          if (firstVideoLecture) break;
        }

        if (targetLecture && targetSection) {
          const isDoc = isDocumentLecture(targetLecture);
          if (!isDoc) {
            activeVideoLecture = targetLecture;
            activeVideoSection = targetSection;
            activeLecture = targetLecture;
            activeSection = targetSection;
            const videoTab: CourseTab = {
              id: 'video',
              title: targetLecture.title,
              type: 'video',
              lecture: targetLecture,
              section: targetSection,
              closeable: false,
            };
            openTabs = [videoTab];
            activeTabId = 'video';
            shouldAutoPlay = autoPlay;
          } else {
            const docTab: CourseTab = {
              id: targetLecture.id,
              title: targetLecture.title,
              type: getLectureType(targetLecture),
              lecture: targetLecture,
              section: targetSection,
              closeable: true,
            };
            if (firstVideoLecture && firstVideoSection) {
              activeVideoLecture = firstVideoLecture;
              activeVideoSection = firstVideoSection;
              const videoTab: CourseTab = {
                id: 'video',
                title: firstVideoLecture.title,
                type: 'video',
                lecture: firstVideoLecture,
                section: firstVideoSection,
                closeable: false,
              };
              openTabs = [videoTab, docTab];
            } else {
              docTab.closeable = false;
              openTabs = [docTab];
            }
            activeTabId = targetLecture.id;
            activeLecture = targetLecture;
            activeSection = targetSection;
            shouldAutoPlay = false;

            if (!currentProgress.completed_lecture_ids?.includes(targetLecture.id)) {
              handleToggleCompleted(targetLecture.id, new MouseEvent('click'));
            }
          }
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
    const isDoc = isDocumentLecture(lecture);

    if (!isDoc) {
      // VIDEO lecture
      if (currentCourse && activeVideoLecture && videoPlayerRef) {
        const cur = videoPlayerRef.getCurrentTime();
        if (cur > 0) {
          handlePositionUpdated(cur);
        }
      }

      shouldAutoPlay = autoPlay;
      activeVideoLecture = lecture;
      activeVideoSection = section;
      activeLecture = lecture;
      activeSection = section;

      const videoTabIdx = openTabs.findIndex((t) => t.id === 'video' || t.type === 'video');
      const videoTab: CourseTab = {
        id: 'video',
        title: lecture.title,
        type: 'video',
        lecture,
        section,
        closeable: false,
      };

      if (videoTabIdx >= 0) {
        openTabs[videoTabIdx] = videoTab;
      } else {
        openTabs = [videoTab, ...openTabs];
      }

      activeTabId = 'video';

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
    } else {
      // DOCUMENT lecture (HTML or PDF)
      activeLecture = lecture;
      activeSection = section;

      const existingTab = openTabs.find((t) => t.id === lecture.id);
      if (existingTab) {
        activeTabId = existingTab.id;
      } else {
        const docTab: CourseTab = {
          id: lecture.id,
          title: lecture.title,
          type: getLectureType(lecture),
          lecture,
          section,
          closeable: true,
        };
        openTabs = [...openTabs, docTab];
        activeTabId = docTab.id;
      }

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

        if (!currentProgress.completed_lecture_ids?.includes(lecture.id)) {
          handleToggleCompleted(lecture.id, new MouseEvent('click'));
        }
      }
    }
  }

  function selectTab(tabId: string) {
    activeTabId = tabId;
    const tab = openTabs.find((t) => t.id === tabId);
    if (tab) {
      activeLecture = tab.lecture;
      activeSection = tab.section;
      if (currentProgress && currentCourse) {
        currentProgress.last_played_lecture_id = tab.lecture.id;
        api.updateSettings({ ...userSettings, active_course_id: currentCourse.id }).catch(() => {});
      }
    }
  }

  function closeTab(tabId: string) {
    const index = openTabs.findIndex((t) => t.id === tabId);
    if (index === -1) return;
    const tabToClose = openTabs[index];
    if (!tabToClose.closeable) return;

    const remainingTabs = openTabs.filter((t) => t.id !== tabId);
    openTabs = remainingTabs;

    if (activeTabId === tabId) {
      let nextTab: CourseTab | undefined;
      if (index > 0 && remainingTabs[index - 1]) {
        nextTab = remainingTabs[index - 1];
      } else if (remainingTabs.length > 0) {
        nextTab = remainingTabs[0];
      }

      if (nextTab) {
        selectTab(nextTab.id);
      } else {
        activeTabId = 'video';
      }
    }
  }

  async function handleToggleCompleted(lectureId: string, event: MouseEvent) {
    event.stopPropagation();
    if (!currentCourse || !currentProgress) return;

    // HTML & PDF documents are always checked and cannot be modified
    for (const sec of currentCourse.sections) {
      for (const lec of sec.lectures) {
        if (lec.id === lectureId) {
          if (isDocumentLecture(lec)) return;
          break;
        }
      }
    }

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
        courses = await api.listCourses();
      }
    } catch (e) {
      console.error('Failed to update progress:', e);
    }
  }

  function handlePositionUpdated(posSec: number) {
    currentVideoTime = posSec;
    if (currentCourse && activeVideoLecture) {
      if (!currentProgress) {
        currentProgress = {
          course_id: currentCourse.id,
          completed_lecture_ids: [],
          last_played_lecture_id: activeVideoLecture.id,
          playback_positions: {},
          notes: [],
        };
      }
      currentProgress.playback_positions[activeVideoLecture.id] = posSec;
      currentProgress.last_played_lecture_id = activeVideoLecture.id;

      api.updatePosition(currentCourse.id, activeVideoLecture.id, posSec).catch(() => {});
    }
  }

  function handleDurationUpdated(durSec: number) {
    if (currentCourse && activeVideoLecture && durSec > 0) {
      activeVideoLecture.duration_seconds = durSec;
      if (activeVideoSection) {
        activeVideoSection.duration_seconds = activeVideoSection.lectures.reduce(
          (acc, l) => acc + (l.duration_seconds || 0),
          0
        );
      }
      api.updateDuration(currentCourse.id, activeVideoLecture.id, durSec).catch(() => {});
    }
  }

  function handleVideoEnded() {
    if (activeVideoLecture && currentProgress && currentCourse) {
      if (!currentProgress.completed_lecture_ids.includes(activeVideoLecture.id)) {
        handleToggleCompleted(activeVideoLecture.id, new MouseEvent('click'));
      }
      if (activeVideoLecture.duration_seconds > 0) {
        handlePositionUpdated(activeVideoLecture.duration_seconds);
      }
      if (userSettings.auto_play_next) {
        playNextLecture(activeVideoLecture);
      }
    }
  }

  function playNextLecture(fromLecture?: Lecture | null) {
    if (!currentCourse) return;
    const baseLecture = fromLecture ?? currentDisplayLecture;
    if (!baseLecture) return;

    let found = false;
    for (const s of currentCourse.sections) {
      for (const l of s.lectures) {
        if (found) {
          selectLecture(l, s, true);
          return;
        }
        if (l.id === baseLecture.id) {
          found = true;
        }
      }
    }
  }

  async function handleSaveNote(text: string, timestamp: number) {
    if (!currentCourse || !currentDisplayLecture) return;
    const exactTime =
      (activeTabId === 'video' || openTabs.length === 0) && videoPlayerRef
        ? videoPlayerRef.getCurrentTime()
        : timestamp;
    try {
      const notes = await api.addNote(
        currentCourse.id,
        currentDisplayLecture.id,
        currentDisplayLecture.title,
        exactTime,
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
    if (activeVideoLecture?.id === lectureId) {
      activeTabId = 'video';
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
        scanProgress = null;
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
      scanProgress = null;
    }
  }

  async function handleRescanCourse(courseId: string, event: MouseEvent) {
    event.stopPropagation();
    try {
      scanProgress = null;
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
      scanProgress = null;
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
          activeVideoLecture = null;
          activeVideoSection = null;
          openTabs = [];
          activeTabId = 'video';
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
      onSelectCourse={(id) => loadCourse(id, false)}
      onImportCourse={handleImportCourse}
      onRescanCourse={handleRescanCourse}
      onDeleteCourse={handleDeleteCourse}
    />
  {:else}
    <!-- Course Player View -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left / Center Player & Details Scrollable Area -->
      <div class="flex-1 flex flex-col min-w-0 overflow-hidden bg-background">
        <!-- Sleek Horizontal Tab Bar -->
        <TabBar
          tabs={openTabs}
          {activeTabId}
          onSelectTab={selectTab}
          onCloseTab={closeTab}
        />

        <main class="flex-1 flex flex-col overflow-y-auto">
          <!-- Video Player Container: stays mounted in DOM so playback position and running state are NOT lost -->
          {#if activeVideoLecture || openTabs.length === 0}
            <div class="w-full flex-col {activeTabId === 'video' || openTabs.length === 0 ? 'flex' : 'hidden'}">
              <VideoPlayer
                bind:this={videoPlayerRef}
                {currentCourse}
                activeLecture={activeVideoLecture}
                activeSection={activeVideoSection}
                {streamingPort}
                {isImporting}
                autoPlay={shouldAutoPlay}
                savedPosition={(activeVideoLecture && currentProgress?.playback_positions?.[activeVideoLecture.id]) || 0}
                notes={currentProgress?.notes?.filter((n) => n.lecture_id === activeVideoLecture?.id) ?? []}
                onTimeUpdate={(t) => (currentVideoTime = t)}
                onPositionUpdated={handlePositionUpdated}
                onDurationUpdated={handleDurationUpdated}
                onVideoEnded={handleVideoEnded}
                onNextLecture={() => playNextLecture(activeVideoLecture)}
                onImportCourse={handleImportCourse}
              />
            </div>
          {/if}

          <!-- Open Document Tabs (HTML & PDF viewers) -->
          {#each openTabs as tab (tab.id)}
            {#if tab.type === 'pdf'}
              <div class="w-full flex-col {activeTabId === tab.id ? 'flex' : 'hidden'}">
                <PdfViewer
                  {currentCourse}
                  activeLecture={tab.lecture}
                  activeSection={tab.section}
                  {streamingPort}
                  isCompleted={currentProgress?.completed_lecture_ids?.includes(tab.lecture.id) ?? false}
                  onToggleCompleted={(e) => handleToggleCompleted(tab.lecture.id, e)}
                  onNextLecture={() => playNextLecture(tab.lecture)}
                />
              </div>
            {:else if tab.type === 'html'}
              <div class="w-full flex-col {activeTabId === tab.id ? 'flex' : 'hidden'}">
                <HtmlViewer
                  {currentCourse}
                  activeLecture={tab.lecture}
                  activeSection={tab.section}
                  {streamingPort}
                  isCompleted={currentProgress?.completed_lecture_ids?.includes(tab.lecture.id) ?? false}
                  onToggleCompleted={(e) => handleToggleCompleted(tab.lecture.id, e)}
                  onNextLecture={() => playNextLecture(tab.lecture)}
                />
              </div>
            {/if}
          {/each}

          <!-- Tabs & Information Below Video / Document Viewer -->
          {#if currentCourse && currentDisplayLecture}
            <CourseTabs
              course={currentCourse}
              stats={currentStats}
              progress={currentProgress}
              activeLecture={currentDisplayLecture}
              activeSection={currentDisplaySection}
              currentTime={currentVideoTime}
              onSaveNote={handleSaveNote}
              onDeleteNote={handleDeleteNote}
              onSeekToNote={handleSeekToNote}
            />
          {/if}
        </main>
      </div>

      <!-- Right Sidebar Drawer -->
      <CourseContentSidebar
        course={currentCourse}
        stats={currentStats}
        progress={currentProgress}
        activeLecture={currentDisplayLecture}
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
        <div class="space-y-1.5 w-full">
          <div class="flex items-center justify-center gap-2">
            <h3 class="text-base font-bold text-foreground">Importing Course</h3>
            {#if scanProgress && scanProgress.total > 0}
              <span class="text-xs font-semibold text-primary px-2 py-0.5 rounded-md bg-primary/10 border border-primary/20 font-mono">
                {Math.round((scanProgress.current / scanProgress.total) * 100)}%
              </span>
            {/if}
          </div>
          <p class="text-xs text-muted-foreground leading-relaxed truncate max-w-full px-1" title={scanProgress?.current_file}>
            {#if scanProgress && scanProgress.total > 0}
              Indexing {scanProgress.current}/{scanProgress.total}: {scanProgress.current_file}
            {:else}
              Scanning directory and indexing videos...
            {/if}
          </p>
        </div>

        <!-- Progress Bar Container -->
        <div class="relative w-full bg-[#22203d] h-2 rounded-full overflow-hidden border border-border/40">
          {#if scanProgress && scanProgress.total > 0}
            <div
              class="h-full bg-primary rounded-full transition-all duration-200 ease-out"
              style="width: {Math.max(4, Math.min(100, Math.round((scanProgress.current / scanProgress.total) * 100)))}%"
            ></div>
          {:else}
            <div class="animate-indeterminate bg-primary"></div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
