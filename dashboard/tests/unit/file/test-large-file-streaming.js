import { describe, it, expect, beforeEach, vi } from 'vitest';
import { FileReaderComponent } from '../../../src/file/file-reader.js';
import { FileReadError } from '../../../src/error.js';

describe('Large File Streaming', () => {
  let fileReader;

  beforeEach(() => {
    fileReader = new FileReaderComponent();
  });

  it('reads small files directly without chunking', async () => {
    const smallFile = new File(['test data'], 'small.arrow', { type: 'application/octet-stream' });

    const buffer = await fileReader.readFile(smallFile, 'small.arrow');

    expect(buffer).toBeInstanceOf(ArrayBuffer);
    expect(buffer.byteLength).toBe(9); // "test data" length
  });

  it('uses chunked reading for large files', async () => {
    // Create a large file (simulate > 10MB)
    const largeData = new Uint8Array(15 * 1024 * 1024); // 15MB
    largeData.fill(42);
    const largeFile = new File([largeData], 'large.arrow', { type: 'application/octet-stream' });

    const startTime = Date.now();
    const buffer = await fileReader.readFile(largeFile, 'large.arrow', {
      chunkSize: 5 * 1024 * 1024, // 5MB chunks
    });
    const duration = Date.now() - startTime;

    expect(buffer).toBeInstanceOf(ArrayBuffer);
    expect(buffer.byteLength).toBe(15 * 1024 * 1024);
    // Should complete without errors
    expect(duration).toBeLessThan(10000); // Should complete in reasonable time
  });

  it('warns about very large files exceeding max size', async () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    try {
      const hugeData = new Uint8Array(250 * 1024 * 1024); // 250MB
      hugeData.fill(42);
      const hugeFile = new File([hugeData], 'huge.arrow', { type: 'application/octet-stream' });

      await fileReader.readFile(hugeFile, 'huge.arrow', {
        maxSize: 200 * 1024 * 1024, // 200MB max
      });

      expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('File huge.arrow is large'));
    } finally {
      consoleSpy.mockRestore();
    }
  });

  it('handles file read errors gracefully', async () => {
    const mockFile = {
      name: 'error.arrow',
      size: 1000,
      slice: vi.fn(() => {
        throw new Error('Read error');
      }),
      arrayBuffer: vi.fn(() => {
        throw new Error('Read error');
      }),
    };

    await expect(fileReader.readFile(mockFile, 'error.arrow')).rejects.toThrow(FileReadError);
  });

  it('reads files in chunks and combines correctly', async () => {
    // Create file with known pattern
    const chunk1 = new Uint8Array(5 * 1024 * 1024).fill(1);
    const chunk2 = new Uint8Array(5 * 1024 * 1024).fill(2);
    const chunk3 = new Uint8Array(3 * 1024 * 1024).fill(3);
    const allChunks = new Uint8Array([...chunk1, ...chunk2, ...chunk3]);
    const testFile = new File([allChunks], 'test.arrow');

    const buffer = await fileReader.readFile(testFile, 'test.arrow', {
      chunkSize: 5 * 1024 * 1024,
    });

    const result = new Uint8Array(buffer);
    expect(result.length).toBe(allChunks.length);
    // Verify data integrity
    expect(result[0]).toBe(1);
    expect(result[5 * 1024 * 1024]).toBe(2);
    expect(result[10 * 1024 * 1024]).toBe(3);
  });

  it('yields to event loop during large file reads', async () => {
    let yieldCount = 0;
    const originalSetTimeout = global.setTimeout;
    global.setTimeout = vi.fn((fn) => {
      yieldCount++;
      return originalSetTimeout(fn, 0);
    });

    try {
      const largeData = new Uint8Array(60 * 1024 * 1024); // 60MB
      largeData.fill(42);
      const largeFile = new File([largeData], 'large.arrow');

      await fileReader.readFile(largeFile, 'large.arrow', {
        chunkSize: 5 * 1024 * 1024,
      });

      // Should yield multiple times (60MB / 5MB = 12 chunks, yields every 10 chunks)
      expect(yieldCount).toBeGreaterThan(0);
    } finally {
      global.setTimeout = originalSetTimeout;
    }
  });
});
