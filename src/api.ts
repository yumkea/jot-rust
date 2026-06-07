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
  return window.__TAURI__.core.invoke("list_notes") as Promise<Note[]>;
}

export async function saveNote(
  id: string,
  title: string,
  content: string
): Promise<{ success: boolean }> {
  return window.__TAURI__.core.invoke("save_note", { id, title, content }) as Promise<{ success: boolean }>;
}

export async function deleteNote(id: string): Promise<{ success: boolean }> {
  return window.__TAURI__.core.invoke("delete_note", { id }) as Promise<{ success: boolean }>;
}

export async function getSettings(): Promise<Settings> {
  return window.__TAURI__.core.invoke("get_settings") as Promise<Settings>;
}

export async function saveSetting(
  key: string,
  value: string
): Promise<{ success: boolean }> {
  return window.__TAURI__.core.invoke("save_setting", { key, value }) as Promise<{ success: boolean }>;
}

export async function getStoragePath(): Promise<string> {
  return window.__TAURI__.core.invoke("get_storage_path") as Promise<string>;
}
