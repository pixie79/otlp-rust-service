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

  /**
   * Read file with incremental/chunked reading for large files
   * For files > 50MB, uses chunked reading to avoid blocking
   * @param {File|FileHandle} fileReference - File reference
   * @param {string} fileNameOverride - Optional file name override
   * @param {Object} options - Options for reading (chunkSize, maxSize)
   * @returns {Promise<ArrayBuffer>} File contents as ArrayBuffer
   */
  async readFile(fileReference, fileNameOverride, options = {}) {
    try {
      const file = await this._getFileInstance(fileReference);
      const chunkSize = options.chunkSize || 10 * 1024 * 1024; // 10MB chunks
      const maxSize = options.maxSize || 200 * 1024 * 1024; // 200MB max
      const fileName = fileNameOverride ?? fileReference.name ?? 'unknown';

      // For small files, read directly
      if (file.size < chunkSize) {
        return await file.arrayBuffer();
      }

      // For large files, warn if exceeding max size
      if (file.size > maxSize) {
        console.warn(
          `File ${fileName} is large (${(file.size / 1024 / 1024).toFixed(2)}MB). Consider splitting into smaller files.`
        );
      }

      // Use chunked reading for large files
      // Note: Arrow IPC files need to be read completely for parsing
      // But we can read in chunks and combine to avoid blocking the main thread
      const chunks = [];
      let offset = 0;

      while (offset < file.size) {
        const chunk = file.slice(offset, offset + chunkSize);
        const chunkBuffer = await chunk.arrayBuffer();
        chunks.push(chunkBuffer);
        offset += chunkSize;

        // Yield to event loop periodically for large files
        if (chunks.length % 10 === 0) {
          await new Promise((resolve) => setTimeout(resolve, 0));
        }
      }

      // Combine chunks into single ArrayBuffer
      const totalLength = chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0);
      const result = new Uint8Array(totalLength);
      let position = 0;

      for (const chunk of chunks) {
        result.set(new Uint8Array(chunk), position);
        position += chunk.byteLength;
      }

      return result.buffer;
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
