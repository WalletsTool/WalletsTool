import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { utils as xlUtils, write } from 'xlsx';

export async function exportWithDialog(data, defaultFilename) {
  const filePath = await save({
    defaultPath: defaultFilename,
    filters: [{ name: 'Excel Files', extensions: ['xlsx'] }]
  });

  if (!filePath) return null;

  const workbook = xlUtils.book_new();
  const worksheet = xlUtils.aoa_to_sheet(data);
  xlUtils.book_append_sheet(workbook, worksheet, 'Sheet1');

  const buffer = write(workbook, { bookType: 'xlsx', type: 'array' });
  const uint8Array = new Uint8Array(buffer);

  await invoke('save_file', { filePath, content: uint8Array });
  return filePath;
}

export function openDirectory(filePath) {
  invoke('open_file_directory', { filePath });
}
