import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

export async function downloadWithDialog(relativePath, defaultFilename) {
  const filePath = await save({
    defaultPath: defaultFilename,
    filters: [{ name: 'Excel Files', extensions: ['xlsx'] }]
  });

  if (!filePath) return null;

  const content = await invoke('read_resource_file', { relativePath });
  await invoke('save_file', { filePath, content: new Uint8Array(content) });
  return filePath;
}

export function openDirectory(filePath) {
  invoke('open_file_directory', { filePath });
}
