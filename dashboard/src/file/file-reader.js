import { FileSystemAPI } from './file-system-api.js';
import { FileReadError } from '../error.js';

const hasWindow = () => typeof window !== 'undefined';

const createFileInput = () => {
  if (!hasWindow()) {
    throw new Error('File selection is not available in this environment.');
  }
  const input = document.createElement('input');
  input.type = 'file';
  input.multiple = true;
  input.accept = '.arrow,.ipc';
  input.style.display = 'none';
  document.body.appendChild(input);
  return input;
};

export class FileReaderComponent {
  constructor() {
    this._selectedDirectory = null;
  }

  async selectDirectory() {
    if (FileSystemAPI.isDirectoryPickerSupported()) {
      this._selectedDirectory = await FileSystemAPI.selectDirectory();
      return this._selectedDirectory;
    }

    const input = createFileInput();
    return new Promise((resolve, reject) => {
      input.onchange = () => {
        const files = Array.from(input.files ?? []);
        input.remove();
        if (!files.length) {
          reject(new Error('No files selected'));
          return;
        }
        this._selectedDirectory = files;
        resolve(files);
      };
      input.click();
    });
  }

  async readFile(fileReference, fileNameOverride) {
    try {
      const file = await this._getFileInstance(fileReference);
      return await file.arrayBuffer();
    } catch (error) {
      throw new FileReadError(fileNameOverride ?? fileReference.name ?? 'unknown', error);
    }
  }

  async listFiles(target = this._selectedDirectory) {
    if (!target) {
      return [];
    }

    if (Array.isArray(target)) {
      return target;
    }

    return FileSystemAPI.iterateDirectory(target);
  }

  async getFileMetadata(fileReference) {
    const file = await this._getFileInstance(fileReference);
    return {
      size: file.size,
      lastModified: file.lastModified,
      name: file.name,
    };
  }

  async _getFileInstance(fileReference) {
    if (!fileReference) {
      throw new Error('File reference is required');
    }

    if (typeof fileReference.getFile === 'function') {
      return fileReference.getFile();
    }

    return fileReference;
  }
}
