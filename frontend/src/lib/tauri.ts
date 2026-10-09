import type { CourseDetail, CourseSummary, Note, UserSettings } from './types';

// Tauri IPC wrapper supporting Tauri v2 and browser mocks
async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const tauri = (window as unknown as {
    __TAURI__?: {
      core?: { invoke: (cmd: string, args?: Record<string, unknown>) => Promise<T> };
      invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
    __TAURI_INTERNALS__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
  });

  if (tauri.__TAURI__?.core?.invoke) {
    return await tauri.__TAURI__.core.invoke(cmd, args);
  } else if (tauri.__TAURI__?.invoke) {
    return await tauri.__TAURI__.invoke(cmd, args);
  } else if (tauri.__TAURI_INTERNALS__?.invoke) {
    return await tauri.__TAURI_INTERNALS__.invoke(cmd, args);
  }

  throw new Error(`Tauri IPC not available for command: ${cmd}`);
}

export const api = {
  getStreamingPort: () => invoke<number>('get_streaming_port'),
  listCourses: () => invoke<CourseSummary[]>('list_courses'),
  getCourse: (courseId: string) => invoke<CourseDetail>('get_course', { courseId }),
  scanCourse: (path: string) => invoke<CourseDetail>('scan_course', { path }),
  rescanCourse: (courseId: string) => invoke<CourseDetail>('rescan_course', { courseId }),
  deleteCourse: (courseId: string) => invoke<boolean>('delete_course', { courseId }),
  updateProgress: (courseId: string, lectureId: string, completed: boolean) =>
    invoke<CourseDetail>('update_progress', { courseId, lectureId, completed }),
  updatePosition: (courseId: string, lectureId: string, positionSeconds: number) =>
    invoke<void>('update_position', { courseId, lectureId, positionSeconds }),
  updateDuration: (courseId: string, lectureId: string, durationSeconds: number) =>
    invoke<void>('update_duration', { courseId, lectureId, durationSeconds }),
  addNote: (
    courseId: string,
    lectureId: string,
    lectureTitle: string,
    timestampSeconds: number,
    text: string
  ) =>
    invoke<Note[]>('add_note', {
      courseId,
      lectureId,
      lectureTitle,
      timestampSeconds,
      text,
    }),
  deleteNote: (courseId: string, noteId: string) =>
    invoke<Note[]>('delete_note', { courseId, noteId }),
  getSettings: () => invoke<UserSettings>('get_settings'),
  updateSettings: (settings: UserSettings) =>
    invoke<UserSettings>('update_settings', { settings }),
  pickFolder: () => invoke<string | null>('pick_folder'),
};

export async function listenToEvent<T>(
  event: string,
  handler: (payload: T) => void
): Promise<() => void> {
  const tauri = (window as unknown as {
    __TAURI__?: {
      event?: { listen: <E>(name: string, cb: (e: { payload: E }) => void) => Promise<() => void> };
    };
  });

  if (tauri.__TAURI__?.event?.listen) {
    return await tauri.__TAURI__.event.listen<T>(event, (e) => handler(e.payload));
  }

  try {
    const { listen } = await import('@tauri-apps/api/event');
    return await listen<T>(event, (e) => handler(e.payload));
  } catch (err) {
    console.warn(`Could not listen to event ${event}:`, err);
    return () => {};
  }
}
