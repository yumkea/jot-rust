import { invoke } from '@tauri-apps/api/core'

export interface Note {
  id: string;
  title: string;
  content: string;
  updated_at: string;
}

export interface Settings {
  [key: string]: string;
}

export async function listNotes(): Promise<Note[]> {
  return invoke("list_notes") as Promise<Note[]>;
}

export async function getNote(id: string): Promise<Note | null> {
  return invoke("get_note", { id }) as Promise<Note | null>;
}

export async function saveNote(
  id: string,
  title: string,
  content: string
): Promise<{ success: boolean }> {
  return invoke("save_note", { id, title, content }) as Promise<{ success: boolean }>;
}

export async function deleteNote(id: string): Promise<{ success: boolean }> {
  return invoke("delete_note", { id }) as Promise<{ success: boolean }>;
}

export async function getSettings(): Promise<Settings> {
  return invoke("get_settings") as Promise<Settings>;
}

export async function saveSetting(
  key: string,
  value: string
): Promise<{ success: boolean }> {
  return invoke("save_setting", { key, value }) as Promise<{ success: boolean }>;
}

export async function getStoragePath(): Promise<string> {
  return invoke("get_storage_path") as Promise<string>;
}

export async function setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
  return invoke("set_always_on_top", { alwaysOnTop }) as Promise<void>;
}

export async function hideWindow(): Promise<void> {
  return invoke("hide_window") as Promise<void>;
}

export async function showWindow(): Promise<void> {
  return invoke("show_window") as Promise<void>;
}

export async function setAutoLaunch(enable: boolean): Promise<void> {
  return invoke("set_auto_launch", { enable }) as Promise<void>;
}

export async function getAutoLaunch(): Promise<boolean> {
  return invoke("get_auto_launch") as Promise<boolean>;
}

export async function updateGlobalShortcut(shortcut: string): Promise<void> {
  return invoke("update_global_shortcut", { shortcut }) as Promise<void>;
}
