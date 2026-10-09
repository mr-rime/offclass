<script lang="ts">
  import type { Course, Lecture, Section } from '../types';
  import Button from './ui/Button.svelte';
  import { FolderPlus, Play, Video } from 'lucide-svelte';

  interface Props {
    currentCourse: Course | null;
    activeLecture: Lecture | null;
    activeSection: Section | null;
    streamingPort: number;
    savedPosition: number;
    onPositionUpdated: (sec: number) => void;
    onDurationUpdated: (sec: number) => void;
    onVideoEnded: () => void;
    onImportCourse: () => void;
  }

  let {
    currentCourse,
    activeLecture,
    activeSection,
    streamingPort,
    savedPosition,
    onPositionUpdated,
    onDurationUpdated,
    onVideoEnded,
    onImportCourse,
  }: Props = $props();

  let videoElement: HTMLVideoElement | null = $state(null);
  let lastSavedTime = 0;

  let streamUrl = $derived(
    currentCourse && activeLecture && streamingPort > 0
      ? `http://127.0.0.1:${streamingPort}/stream/${currentCourse.id}/${activeLecture.id}`
      : ''
  );

  $effect(() => {
    if (videoElement && streamUrl) {
      videoElement.src = streamUrl;
      videoElement.load();
    }
  });

  function handleLoadedMetadata() {
    if (!videoElement) return;
    if (savedPosition > 0 && savedPosition < videoElement.duration - 2) {
      videoElement.currentTime = savedPosition;
    }
    if (videoElement.duration && (!activeLecture?.duration_seconds || activeLecture.duration_seconds === 0)) {
      onDurationUpdated(Math.round(videoElement.duration));
    }
    videoElement.play().catch(() => {});
  }

  function handleTimeUpdate() {
    if (!videoElement) return;
    const now = Date.now();
    if (now - lastSavedTime > 4000) {
      lastSavedTime = now;
      onPositionUpdated(videoElement.currentTime);
    }
  }

  export function seekTo(seconds: number) {
    if (videoElement) {
      videoElement.currentTime = seconds;
      videoElement.play().catch(() => {});
    }
  }

  export function getCurrentTime(): number {
    return videoElement ? videoElement.currentTime : 0;
  }
</script>

<div class="w-full bg-black relative flex items-center justify-center min-h-[440px] max-h-[calc(100vh-280px)] shrink-0 overflow-hidden shadow-md">
  {#if activeLecture && streamUrl}
    <video
      bind:this={videoElement}
      controls
      autoplay
      playsinline
      class="w-full h-full max-h-[calc(100vh-280px)] object-contain outline-none focus:outline-none"
      onloadedmetadata={handleLoadedMetadata}
      ontimeupdate={handleTimeUpdate}
      onended={onVideoEnded}
    >
      <track kind="captions" />
    </video>
  {:else}
    <div class="flex flex-col items-center justify-center p-12 text-center gap-4 max-w-md">
      <div class="w-16 h-16 rounded-full bg-[#a435f0]/10 flex items-center justify-center text-[#a435f0] shadow-inner">
        <Video class="w-8 h-8" />
      </div>
      <div class="text-xl font-bold text-white">No Video Selected</div>
      <p class="text-sm text-zinc-400">
        Import a course folder or select a lecture from the right sidebar to start watching offline.
      </p>
      <Button variant="purple" size="default" class="mt-2 gap-2" onclick={onImportCourse}>
        <FolderPlus class="w-4 h-4" />
        <span>Select Course Folder</span>
      </Button>
    </div>
  {/if}
</div>
