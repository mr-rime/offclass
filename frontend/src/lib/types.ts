export interface Lecture {
  id: string;
  title: string;
  file_name: string;
  relative_path: string;
  absolute_path: string;
  duration_seconds: number;
  order: number;
  item_type?: 'video' | 'html';
}

export interface Section {
  id: string;
  title: string;
  order: number;
  lectures: Lecture[];
  duration_seconds: number;
}

export interface CourseStats {
  total_sections: number;
  total_lectures: number;
  total_duration_seconds: number;
  completed_lectures: number;
  progress_percent: number;
}

export interface Note {
  id: string;
  lecture_id: string;
  lecture_title: string;
  timestamp_seconds: number;
  text: string;
  created_at: string;
}

export interface CourseProgress {
  course_id: string;
  completed_lecture_ids: string[];
  last_played_lecture_id: string | null;
  playback_positions: Record<string, number>;
  notes: Note[];
}

export interface Course {
  id: string;
  title: string;
  root_path: string;
  sections: Section[];
  created_at: string;
  updated_at: string;
}

export interface CourseDetail {
  course: Course;
  progress: CourseProgress;
  stats: CourseStats;
}

export interface CourseSummary {
  id: string;
  title: string;
  root_path: string;
  stats: CourseStats;
  last_played_lecture_id: string | null;
  updated_at: string;
}

export interface UserSettings {
  theme: 'dark' | 'light';
  auto_play_next: boolean;
  playback_speed: number;
  volume: number;
  active_course_id: string | null;
}

export interface ScanProgress {
  current: number;
  total: number;
  current_file: string;
  phase: string;
}

