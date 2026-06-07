import { invoke } from "@tauri-apps/api/core";

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
  return invoke("list_notes");
}

export async function saveNote(
  id: string,
  title: string,
  content: string
): Promise<{ success: boolean }> {
  return invoke("save_note", { id, title, content });
}

export async function deleteNote(id: string): Promise<{ success: boolean }> {
  return invoke("delete_note", { id });
}

export async function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export async function saveSetting(
  key: string,
  value: string
): Promise<{ success: boolean }> {
  return invoke("save_setting", { key, value });
}

export async function getStoragePath(): Promise<string> {
  return invoke("get_storage_path");
}
