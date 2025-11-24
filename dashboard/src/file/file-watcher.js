export class FileWatcher {
  constructor(fileReader) {
    this.fileReader = fileReader;
    this.intervalId = null;
    this.directory = null;
    this.intervalMs = 1_000;
    this.knownFiles = new Map(); // File metadata cache
    this.metadataCache = new Map(); // Cache for file metadata to avoid redundant reads
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
      // Use cached metadata if available to avoid redundant file reads
      const cacheKey = fileHandle.name || (await this.fileReader.getFileMetadata(fileHandle)).name;
      let metadata = this.metadataCache.get(cacheKey);

      if (!metadata) {
        metadata = await this.fileReader.getFileMetadata(fileHandle);
        this.metadataCache.set(cacheKey, metadata);
      }

      const key = metadata.name;
      const previous = this.knownFiles.get(key);
      seen.add(key);

      if (!previous) {
        this.knownFiles.set(key, metadata);
        changedFiles.push({ fileHandle, metadata, change: 'new' });
        this.onNewFile(fileHandle, metadata);
        continue;
      }

      // Check if file changed (size or modification time)
      if (previous.size !== metadata.size || previous.lastModified !== metadata.lastModified) {
        // Update cache with new metadata
        this.metadataCache.set(cacheKey, metadata);
        this.knownFiles.set(key, metadata);
        changedFiles.push({ fileHandle, metadata, change: 'modified' });
        this.onFileChanged(fileHandle, metadata);
      }
    }

    // Clean up removed files
    for (const key of this.knownFiles.keys()) {
      if (!seen.has(key)) {
        this.knownFiles.delete(key);
        this.metadataCache.delete(key);
      }
    }

    return changedFiles;
  }

  /**
   * Clear metadata cache (useful for memory management)
   */
  clearCache() {
    this.metadataCache.clear();
  }
}
