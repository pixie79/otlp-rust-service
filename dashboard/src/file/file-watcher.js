export class FileWatcher {
  constructor(fileReader) {
    this.fileReader = fileReader;
    this.intervalId = null;
    this.directory = null;
    this.intervalMs = 1_000;
    this.knownFiles = new Map();
    this.onNewFile = () => {};
    this.onFileChanged = () => {};
  }

  startWatching(directoryHandleOrFiles, intervalMs = 1_000) {
    this.stopWatching();
    this.directory = directoryHandleOrFiles;
    this.intervalMs = intervalMs;
    this.intervalId = setInterval(() => {
      this.checkForChanges().catch((error) => {
        console.error('FileWatcher.poll error', error);
      });
    }, this.intervalMs);
  }

  stopWatching() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  async checkForChanges() {
    if (!this.directory) {
      return [];
    }

    const files = await this.fileReader.listFiles(this.directory);
    const changedFiles = [];
    const seen = new Set();

    for (const fileHandle of files) {
      const metadata = await this.fileReader.getFileMetadata(fileHandle);
      const key = metadata.name;
      const previous = this.knownFiles.get(key);
      seen.add(key);

      if (!previous) {
        this.knownFiles.set(key, metadata);
        changedFiles.push({ fileHandle, metadata, change: 'new' });
        this.onNewFile(fileHandle, metadata);
        continue;
      }

      if (previous.size !== metadata.size || previous.lastModified !== metadata.lastModified) {
        this.knownFiles.set(key, metadata);
        changedFiles.push({ fileHandle, metadata, change: 'modified' });
        this.onFileChanged(fileHandle, metadata);
      }
    }

    for (const key of this.knownFiles.keys()) {
      if (!seen.has(key)) {
        this.knownFiles.delete(key);
      }
    }

    return changedFiles;
  }
}
