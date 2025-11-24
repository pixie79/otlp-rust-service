const hasWindow = () => typeof window !== 'undefined';

export class FileSystemAPI {
  static isDirectoryPickerSupported() {
    return hasWindow() && typeof window.showDirectoryPicker === 'function';
  }

  static async selectDirectory() {
    if (!FileSystemAPI.isDirectoryPickerSupported()) {
      throw new Error('File System Access API is not supported in this browser.');
    }
    return window.showDirectoryPicker({
      mode: 'read',
    });
  }

  static async iterateDirectory(directoryHandle) {
    if (!directoryHandle || typeof directoryHandle.values !== 'function') {
      return [];
    }

    const files = [];
    for await (const handle of directoryHandle.values()) {
      if (handle.kind === 'file') {
        files.push(handle);
      }
    }
    return files;
  }
}
