<script lang="ts">
  import type { Course, Lecture, Section, Note } from '../types';
  import { formatTime } from '../utils';
  import Button from './ui/Button.svelte';
  import {
    Play,
    Pause,
    RotateCcw,
    RotateCw,
    Volume2,
    Volume1,
    VolumeX,
    Maximize,
    Minimize,
    PictureInPicture2,
    SkipForward,
    Settings,
    Check,
    FolderPlus,
    Loader2,
  } from 'lucide-svelte';
  import { onDestroy, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    currentCourse: Course | null;
    activeLecture: Lecture | null;
    activeSection: Section | null;
    streamingPort: number;
    savedPosition: number;
    autoPlay?: boolean;
    notes?: Note[];
    onPositionUpdated: (sec: number) => void;
    onDurationUpdated: (sec: number) => void;
    onVideoEnded: () => void;
    onNextLecture?: () => void;
    onImportCourse: () => void;
  }

  let {
    currentCourse,
    activeLecture,
    activeSection,
    streamingPort,
    savedPosition,
    autoPlay = false,
    notes = [],
    onPositionUpdated,
    onDurationUpdated,
    onVideoEnded,
    onNextLecture,
    onImportCourse,
  }: Props = $props();

  // Elements & State
  let playerContainer: HTMLElement | null = $state(null);
  let videoElement: HTMLVideoElement | null = $state(null);
  let scrubberElement: HTMLElement | null = $state(null);

  // Playback state
  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let bufferedEnd = $state(0);
  let volume = $state(1.0);
  let isMuted = $state(false);
  let playbackRate = $state(1.0);
  let isFullscreen = $state(false);
  let isWaiting = $state(false);
  let showRemainingTime = $state(false);

  // UI / Controls Visibility state
  let controlsVisible = $state(true);
  let hideControlsTimer: ReturnType<typeof setTimeout> | null = null;
  let showSpeedMenu = $state(false);
  let isVolumeHovered = $state(false);

  // Scrubbing & Hover Preview
  let isScrubbing = $state(false);
  let hoverTime = $state<number | null>(null);
  let hoverPosPct = $state(0);

  // Center Flash Feedback Animation
  let flashAction = $state<{ type: 'play' | 'pause' | 'rewind' | 'forward' | 'volume' | 'speed'; text?: string } | null>(null);
  let flashTimer: ReturnType<typeof setTimeout> | null = null;
  let lastClickTime = 0;
  let singleClickTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSavedTime = 0;

  const speedOptions = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

  let streamUrl = $derived(
    currentCourse && activeLecture && streamingPort > 0
      ? `http://127.0.0.1:${streamingPort}/stream/${currentCourse.id}/${activeLecture.id}`
      : ''
  );

  let currentProgressPct = $derived(
    duration > 0 ? Math.min(100, Math.max(0, (currentTime / duration) * 100)) : 0
  );

  let bufferedProgressPct = $derived(
    duration > 0 ? Math.min(100, Math.max(0, (bufferedEnd / duration) * 100)) : 0
  );

  // Stream URL update
  $effect(() => {
    if (videoElement && streamUrl) {
      videoElement.src = streamUrl;
      videoElement.load();
    }
  });

  function triggerFlash(type: 'play' | 'pause' | 'rewind' | 'forward' | 'volume' | 'speed', text?: string) {
    if (flashTimer) clearTimeout(flashTimer);
    flashAction = { type, text };
    flashTimer = setTimeout(() => {
      flashAction = null;
    }, 600);
  }

  function flushPosition() {
    if (videoElement && !isNaN(videoElement.currentTime) && videoElement.currentTime >= 0) {
      onPositionUpdated(videoElement.currentTime);
    }
  }

  function handleLoadedMetadata() {
    if (!videoElement) return;
    duration = videoElement.duration || activeLecture?.duration_seconds || 0;

    if (savedPosition > 0 && (!videoElement.duration || savedPosition < videoElement.duration - 0.5)) {
      videoElement.currentTime = savedPosition;
      currentTime = savedPosition;
    } else {
      currentTime = videoElement.currentTime || 0;
    }

    if (videoElement.duration && !isNaN(videoElement.duration) && videoElement.duration > 0) {
      const exact = Math.round(videoElement.duration);
      if (activeLecture?.duration_seconds !== exact) {
        onDurationUpdated(exact);
      }
    }

    // Set playback rate & volume
    videoElement.playbackRate = playbackRate;
    videoElement.volume = isMuted ? 0 : volume;

    if (autoPlay) {
      videoElement.play().then(() => {
        isPlaying = true;
      }).catch(() => {});
    } else {
      videoElement.pause();
      isPlaying = false;
    }
  }

  function handleTimeUpdate() {
    if (!videoElement || isScrubbing) return;
    currentTime = videoElement.currentTime;

    // Calculate buffer
    if (videoElement.buffered.length > 0) {
      bufferedEnd = videoElement.buffered.end(videoElement.buffered.length - 1);
    }

    const now = Date.now();
    if (now - lastSavedTime > 1500) {
      lastSavedTime = now;
      onPositionUpdated(currentTime);
    }
  }

  function togglePlayPause() {
    if (!videoElement) return;
    if (videoElement.paused) {
      videoElement.play().then(() => {
        isPlaying = true;
        triggerFlash('play');
      }).catch(() => {});
    } else {
      videoElement.pause();
      isPlaying = false;
      flushPosition();
      triggerFlash('pause');
    }
    showControlsTemporarily();
  }

  function handleVideoClick(event: MouseEvent) {
    // If click was inside controls overlay, don't trigger center play/pause
    if ((event.target as HTMLElement)?.closest('.controls-bar')) return;

    const now = Date.now();
    if (now - lastClickTime < 280) {
      // Double click -> toggle fullscreen
      if (singleClickTimer) clearTimeout(singleClickTimer);
      toggleFullscreen();
      lastClickTime = 0;
    } else {
      lastClickTime = now;
      singleClickTimer = setTimeout(() => {
        togglePlayPause();
      }, 280);
    }
  }

  function handleSkip(seconds: number) {
    if (!videoElement) return;
    const target = Math.max(0, Math.min(duration, videoElement.currentTime + seconds));
    videoElement.currentTime = target;
    currentTime = target;
    flushPosition();
    triggerFlash(seconds > 0 ? 'forward' : 'rewind', `${seconds > 0 ? '+' : ''}${seconds}s`);
    showControlsTemporarily();
  }

  function handleVolumeChange(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    volume = val;
    isMuted = val === 0;
    if (videoElement) {
      videoElement.volume = volume;
      videoElement.muted = isMuted;
    }
    triggerFlash('volume', `${Math.round(volume * 100)}%`);
  }

  function toggleMute() {
    isMuted = !isMuted;
    if (videoElement) {
      videoElement.muted = isMuted;
      videoElement.volume = isMuted ? 0 : volume || 0.5;
    }
    triggerFlash('volume', isMuted ? 'Muted' : `${Math.round(volume * 100)}%`);
    showControlsTemporarily();
  }

  function setSpeed(rate: number) {
    playbackRate = rate;
    if (videoElement) {
      videoElement.playbackRate = rate;
    }
    showSpeedMenu = false;
    triggerFlash('speed', `${rate}x`);
    showControlsTemporarily();
  }

  async function togglePiP() {
    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture();
      } else if (videoElement) {
        await videoElement.requestPictureInPicture();
      }
    } catch (e) {
      console.warn('PiP failed:', e);
    }
  }

  async function toggleFullscreen() {
    if (!playerContainer) return;
    try {
      if (!document.fullscreenElement) {
        await playerContainer.requestFullscreen();
        isFullscreen = true;
      } else {
        await document.exitFullscreen();
        isFullscreen = false;
      }
    } catch (e) {
      console.warn('Fullscreen toggle failed:', e);
    }
  }

  // Scrubber Dragging / Hover Logic
  function getScrubTimeFromEvent(e: MouseEvent): number {
    if (!scrubberElement || duration <= 0) return 0;
    const rect = scrubberElement.getBoundingClientRect();
    const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    return pos * duration;
  }

  function handleScrubberMouseDown(e: MouseEvent) {
    isScrubbing = true;
    const targetTime = getScrubTimeFromEvent(e);
    currentTime = targetTime;
    if (videoElement) {
      videoElement.currentTime = targetTime;
    }

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (isScrubbing) {
        const time = getScrubTimeFromEvent(moveEvent);
        currentTime = time;
        if (videoElement) {
          videoElement.currentTime = time;
        }
      }
    };

    const handleMouseUp = () => {
      isScrubbing = false;
      flushPosition();
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleScrubberMouseMove(e: MouseEvent) {
    if (!scrubberElement || duration <= 0) return;
    const rect = scrubberElement.getBoundingClientRect();
    const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    hoverPosPct = pos * 100;
    hoverTime = pos * duration;
  }

  function handleScrubberMouseLeave() {
    hoverTime = null;
  }

  // Auto-hide controls overlay
  function showControlsTemporarily() {
    controlsVisible = true;
    if (hideControlsTimer) clearTimeout(hideControlsTimer);
    if (isPlaying && !showSpeedMenu && !isScrubbing) {
      hideControlsTimer = setTimeout(() => {
        controlsVisible = false;
      }, 2600);
    }
  }

  function handleMouseMove() {
    showControlsTemporarily();
  }

  function handleMouseLeave() {
    if (isPlaying && !showSpeedMenu && !isScrubbing) {
      controlsVisible = false;
    }
  }

  // Keyboard Shortcuts
  function handleKeyDown(e: KeyboardEvent) {
    // Ignore if inside text input/textarea
    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    if (tag === 'input' || tag === 'textarea') return;

    switch (e.key.toLowerCase()) {
      case ' ':
      case 'k':
        e.preventDefault();
        togglePlayPause();
        break;
      case 'arrowleft':
      case 'j':
        e.preventDefault();
        handleSkip(-10);
        break;
      case 'arrowright':
      case 'l':
        e.preventDefault();
        handleSkip(10);
        break;
      case 'arrowup':
        e.preventDefault();
        volume = Math.min(1, volume + 0.1);
        if (videoElement) videoElement.volume = volume;
        isMuted = false;
        triggerFlash('volume', `${Math.round(volume * 100)}%`);
        showControlsTemporarily();
        break;
      case 'arrowdown':
        e.preventDefault();
        volume = Math.max(0, volume - 0.1);
        if (videoElement) videoElement.volume = volume;
        isMuted = volume === 0;
        triggerFlash('volume', `${Math.round(volume * 100)}%`);
        showControlsTemporarily();
        break;
      case 'm':
        e.preventDefault();
        toggleMute();
        break;
      case 'f':
        e.preventDefault();
        toggleFullscreen();
        break;
      case '>':
        if (e.shiftKey) {
          e.preventDefault();
          const currIdx = speedOptions.indexOf(playbackRate);
          if (currIdx < speedOptions.length - 1) setSpeed(speedOptions[currIdx + 1]);
        }
        break;
      case '<':
        if (e.shiftKey) {
          e.preventDefault();
          const currIdx = speedOptions.indexOf(playbackRate);
          if (currIdx > 0) setSpeed(speedOptions[currIdx - 1]);
        }
        break;
      case 'n':
        if (onNextLecture) {
          e.preventDefault();
          onNextLecture();
        }
        break;
    }
  }

  onMount(() => {
    const handleFullscreenChange = () => {
      isFullscreen = !!document.fullscreenElement;
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    window.addEventListener('keydown', handleKeyDown);

    const handleBeforeUnload = () => {
      flushPosition();
    };
    window.addEventListener('beforeunload', handleBeforeUnload);
    window.addEventListener('pagehide', handleBeforeUnload);

    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('beforeunload', handleBeforeUnload);
      window.removeEventListener('pagehide', handleBeforeUnload);
      if (hideControlsTimer) clearTimeout(hideControlsTimer);
      if (flashTimer) clearTimeout(flashTimer);
      if (singleClickTimer) clearTimeout(singleClickTimer);
    };
  });

  onDestroy(() => {
    flushPosition();
  });

  export function seekTo(seconds: number) {
    if (videoElement) {
      videoElement.currentTime = seconds;
      currentTime = seconds;
      flushPosition();
      videoElement.play().then(() => {
        isPlaying = true;
      }).catch(() => {});
    }
  }

  export function getCurrentTime(): number {
    return videoElement ? videoElement.currentTime : 0;
  }
</script>

<!-- Custom Video Player Container -->
<div
  bind:this={playerContainer}
  class="w-full bg-[#0a0a10] relative flex items-center justify-center min-h-[460px] max-h-[calc(100vh-220px)] shrink-0 overflow-hidden shadow-2xl select-none group font-sans {isFullscreen ? 'h-screen max-h-screen' : ''} {!controlsVisible && isPlaying ? 'cursor-none' : ''}"
  onmousemove={handleMouseMove}
  onmouseleave={handleMouseLeave}
  role="region"
  aria-label="Video Player"
>
  {#if activeLecture && streamUrl}
    <!-- HTML5 Video Element (Clean headless engine) -->
    <video
      bind:this={videoElement}
      playsinline
      class="w-full h-full object-contain outline-none focus:outline-none"
      onloadedmetadata={handleLoadedMetadata}
      ontimeupdate={handleTimeUpdate}
      onplay={() => (isPlaying = true)}
      onpause={() => {
        isPlaying = false;
        flushPosition();
      }}
      onwaiting={() => (isWaiting = true)}
      onplaying={() => (isWaiting = false)}
      onended={() => {
        isPlaying = false;
        onVideoEnded();
      }}
      onclick={handleVideoClick}
    >
      <track kind="captions" />
    </video>

    <!-- Center Buffering Spinner -->
    {#if isWaiting}
      <div
        class="absolute inset-0 flex items-center justify-center pointer-events-none z-20"
        in:fade={{ duration: 150 }}
      >
        <div class="w-16 h-16 rounded-full bg-black/60 backdrop-blur-md flex items-center justify-center border border-white/10 shadow-2xl">
          <Loader2 class="w-8 h-8 text-primary animate-spin" />
        </div>
      </div>
    {/if}

    <!-- Center Action Flash Animation Pill -->
    {#if flashAction}
      <div
        class="absolute inset-0 flex items-center justify-center pointer-events-none z-30"
        in:scale={{ start: 0.8, duration: 150, easing: cubicOut }}
        out:fade={{ duration: 250 }}
      >
        <div class="px-5 py-3.5 rounded-2xl bg-black/75 backdrop-blur-md border border-white/15 flex items-center gap-2.5 text-white shadow-2xl">
          {#if flashAction.type === 'play'}
            <Play class="w-6 h-6 fill-current text-primary" />
          {:else if flashAction.type === 'pause'}
            <Pause class="w-6 h-6 fill-current text-primary" />
          {:else if flashAction.type === 'rewind'}
            <RotateCcw class="w-6 h-6 text-primary" />
          {:else if flashAction.type === 'forward'}
            <RotateCw class="w-6 h-6 text-primary" />
          {:else if flashAction.type === 'volume'}
            <Volume2 class="w-6 h-6 text-primary" />
          {:else if flashAction.type === 'speed'}
            <Settings class="w-6 h-6 text-primary" />
          {/if}
          {#if flashAction.text}
            <span class="text-sm font-bold tracking-wide">{flashAction.text}</span>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Top Floating Title Bar (when hovered) -->
    <div
      class="absolute top-0 left-0 right-0 p-5 bg-gradient-to-b from-black/85 via-black/40 to-transparent flex items-center justify-between z-20 transition-opacity duration-300 pointer-events-none {controlsVisible || !isPlaying ? 'opacity-100' : 'opacity-0'}"
    >
      <div class="flex items-center gap-3 min-w-0">
        <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-primary/20 text-primary border border-primary/30 shrink-0">
          {activeSection?.title ?? 'Module'}
        </span>
        <h2 class="text-sm font-semibold text-white/90 truncate drop-shadow-md">
          {activeLecture.order}. {activeLecture.title}
        </h2>
      </div>
    </div>

    <!-- Bottom Controls Overlay -->
    <div
      class="controls-bar absolute bottom-0 left-0 right-0 px-4 pb-3.5 pt-10 bg-gradient-to-t from-black/95 via-black/60 to-transparent z-20 transition-all duration-300 flex flex-col gap-2 {controlsVisible || !isPlaying ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2 pointer-events-none'}"
    >
      <!-- Timeline Scrubber Track Container -->
      <div
        bind:this={scrubberElement}
        class="group/track relative w-full h-5 flex items-center cursor-pointer select-none"
        onmousedown={handleScrubberMouseDown}
        onmousemove={handleScrubberMouseMove}
        onmouseleave={handleScrubberMouseLeave}
        role="slider"
        tabindex="0"
        aria-valuenow={currentTime}
        aria-valuemin="0"
        aria-valuemax={duration}
        aria-label="Seek timeline"
      >
        <!-- Hover Time Preview Tooltip -->
        {#if hoverTime !== null}
          <div
            class="absolute bottom-6 -translate-x-1/2 px-2 py-1 rounded bg-black/90 backdrop-blur-md border border-white/20 text-white text-[11px] font-mono font-semibold shadow-xl pointer-events-none z-30"
            style="left: {hoverPosPct}%"
            in:scale={{ start: 0.85, duration: 120 }}
          >
            {formatTime(hoverTime)}
          </div>
        {/if}

        <!-- Track Background -->
        <div class="w-full h-1.5 group-hover/track:h-2 bg-white/20 rounded-full relative overflow-hidden transition-all duration-200">
          <!-- Buffered Bar -->
          <div
            class="absolute top-0 left-0 bottom-0 bg-white/30 rounded-full transition-all duration-150"
            style="width: {bufferedProgressPct}%"
          ></div>
          <!-- Played Bar -->
          <div
            class="absolute top-0 left-0 bottom-0 bg-gradient-to-r from-primary to-[#8c52ff] rounded-full shadow-sm"
            style="width: {currentProgressPct}%"
          ></div>
        </div>

        <!-- Note / Bookmark markers on timeline -->
        {#if notes && notes.length > 0 && duration > 0}
          {#each notes as note (note.id)}
            {@const notePos = Math.min(100, Math.max(0, (note.timestamp_seconds / duration) * 100))}
            <div
              class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 rounded-full bg-amber-400 ring-2 ring-black/80 hover:scale-150 transition-transform cursor-pointer z-10"
              style="left: {notePos}%"
              title="{formatTime(note.timestamp_seconds)} - {note.text}"
            ></div>
          {/each}
        {/if}

        <!-- Scrubber Thumb -->
        <div
          class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 rounded-full bg-white ring-4 ring-primary/40 shadow-xl transition-transform duration-100 pointer-events-none {isScrubbing ? 'scale-125' : 'scale-0 group-hover/track:scale-100'}"
          style="left: {currentProgressPct}%"
        ></div>
      </div>

      <!-- Bottom Buttons & Indicators Row -->
      <div class="flex items-center justify-between gap-3 text-white">
        <!-- Left Controls (Play, Skip, Next, Volume, Time) -->
        <div class="flex items-center gap-2 sm:gap-3">
          <!-- Play / Pause Button -->
          <button
            class="w-9 h-9 rounded-lg hover:bg-white/15 flex items-center justify-center text-white active:scale-95 transition-all cursor-pointer"
            onclick={togglePlayPause}
            title="{isPlaying ? 'Pause (Space)' : 'Play (Space)'}"
          >
            {#if isPlaying}
              <Pause class="w-5 h-5 fill-current" />
            {:else}
              <Play class="w-5 h-5 fill-current" />
            {/if}
          </button>

          <!-- Rewind 10s -->
          <button
            class="w-8 h-8 rounded-lg hover:bg-white/15 flex items-center justify-center text-white/90 hover:text-white active:scale-95 transition-all cursor-pointer"
            onclick={() => handleSkip(-10)}
            title="Rewind 10s (← / J)"
          >
            <RotateCcw class="w-4 h-4" />
          </button>

          <!-- Forward 10s -->
          <button
            class="w-8 h-8 rounded-lg hover:bg-white/15 flex items-center justify-center text-white/90 hover:text-white active:scale-95 transition-all cursor-pointer"
            onclick={() => handleSkip(10)}
            title="Forward 10s (→ / L)"
          >
            <RotateCw class="w-4 h-4" />
          </button>

          <!-- Next Lecture (if available) -->
          {#if onNextLecture}
            <button
              class="w-8 h-8 rounded-lg hover:bg-white/15 flex items-center justify-center text-white/90 hover:text-white active:scale-95 transition-all cursor-pointer"
              onclick={onNextLecture}
              title="Next Lecture (N)"
            >
              <SkipForward class="w-4 h-4 fill-current" />
            </button>
          {/if}

          <!-- Volume Controls with Hover Slider -->
          <div
            class="flex items-center gap-1.5 group/vol relative py-1"
            onmouseenter={() => (isVolumeHovered = true)}
            onmouseleave={() => (isVolumeHovered = false)}
            role="group"
            aria-label="Volume controls"
          >
            <button
              class="w-8 h-8 rounded-lg hover:bg-white/15 flex items-center justify-center text-white/90 hover:text-white active:scale-95 transition-all cursor-pointer"
              onclick={toggleMute}
              title="{isMuted ? 'Unmute (M)' : 'Mute (M)'}"
            >
              {#if isMuted || volume === 0}
                <VolumeX class="w-4 h-4 text-destructive" />
              {:else if volume < 0.5}
                <Volume1 class="w-4 h-4" />
              {:else}
                <Volume2 class="w-4 h-4" />
              {/if}
            </button>

            <!-- Smooth Expandable Slider -->
            <div class="w-0 group-hover/vol:w-20 transition-all duration-200 overflow-hidden flex items-center">
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={isMuted ? 0 : volume}
                oninput={handleVolumeChange}
                class="w-18 h-1.5 bg-white/25 rounded-lg appearance-none cursor-pointer accent-primary"
                aria-label="Volume slider"
              />
            </div>
          </div>

          <!-- Time Display -->
          <button
            class="text-xs font-mono font-medium text-white/85 hover:text-white transition-colors cursor-pointer px-1 py-0.5 rounded hover:bg-white/10 ml-1"
            onclick={() => (showRemainingTime = !showRemainingTime)}
            title="Click to toggle remaining time"
          >
            {#if showRemainingTime && duration > 0}
              <span>-{formatTime(Math.max(0, duration - currentTime))}</span>
            {:else}
              <span>{formatTime(currentTime)} / {formatTime(duration)}</span>
            {/if}
          </button>
        </div>

        <!-- Right Controls (Speed, PiP, Fullscreen) -->
        <div class="flex items-center gap-1 sm:gap-2 relative">
          <!-- Playback Speed Button & Popover -->
          <div class="relative">
            <button
              class="px-2.5 py-1 rounded-lg hover:bg-white/15 text-xs font-semibold text-white/90 hover:text-white transition-all cursor-pointer flex items-center gap-1 border border-white/10"
              onclick={() => (showSpeedMenu = !showSpeedMenu)}
              title="Playback speed"
            >
              <span>{playbackRate}x</span>
            </button>

            <!-- Speed Popover Menu -->
            {#if showSpeedMenu}
              <div
                class="absolute bottom-11 right-0 bg-[#161528] border border-border/90 rounded-xl p-1.5 shadow-2xl z-40 min-w-[110px] space-y-0.5"
                in:scale={{ start: 0.9, duration: 150, easing: cubicOut }}
                out:fade={{ duration: 100 }}
              >
                <div class="text-[10px] font-bold text-muted-foreground uppercase px-2.5 py-1 tracking-wider border-b border-border/40 mb-1">
                  Speed
                </div>
                {#each speedOptions as rate}
                  <button
                    class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {playbackRate === rate ? 'bg-primary text-white font-semibold' : 'text-muted-foreground hover:text-foreground hover:bg-[#201e38]'}"
                    onclick={() => setSpeed(rate)}
                  >
                    <span>{rate}x</span>
                    {#if playbackRate === rate}
                      <Check class="w-3 h-3 stroke-[3]" />
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Picture in Picture -->
          <button
            class="w-8 h-8 rounded-lg hover:bg-white/15 flex items-center justify-center text-white/90 hover:text-white active:scale-95 transition-all cursor-pointer"
            onclick={togglePiP}
            title="Picture-in-Picture"
          >
            <PictureInPicture2 class="w-4 h-4" />
          </button>

          <!-- Fullscreen Toggle -->
          <button
            class="w-8 h-8 rounded-lg hover:bg-white/15 flex items-center justify-center text-white/90 hover:text-white active:scale-95 transition-all cursor-pointer"
            onclick={toggleFullscreen}
            title="{isFullscreen ? 'Exit Fullscreen (F)' : 'Fullscreen (F)'}"
          >
            {#if isFullscreen}
              <Minimize class="w-4 h-4" />
            {:else}
              <Maximize class="w-4 h-4" />
            {/if}
          </button>
        </div>
      </div>
    </div>
  {:else}
    <!-- Empty State when no video is loaded -->
    <div class="flex flex-col items-center justify-center p-12 text-center gap-4 max-w-md">
      <div class="w-16 h-16 rounded-xl bg-[#22203d] border border-border/80 flex items-center justify-center overflow-hidden shadow-md">
        <img src="/logo.png" alt="OffClass" class="w-full h-full object-cover rounded-xl" />
      </div>
      <div class="text-xl font-bold text-white">No Video Selected</div>
      <p class="text-sm text-[#827f9e]">
        Import a course folder or select a lecture from the right sidebar to start watching offline.
      </p>
      <Button variant="purple" size="default" class="mt-2 gap-2" onclick={onImportCourse}>
        <FolderPlus class="w-4 h-4" />
        <span>Select Course Folder</span>
      </Button>
    </div>
  {/if}
</div>
