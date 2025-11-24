/**
 * Performance benchmark for file streaming
 * Measures file read performance with different file sizes
 */

import { FileReaderComponent } from '../../src/file/file-reader.js';

async function benchmarkFileStreaming() {
  const fileReader = new FileReaderComponent();
  const results = [];

  // Test with different file sizes (simulated)
  const fileSizes = [1024, 10240, 102400, 1048576]; // 1KB, 10KB, 100KB, 1MB

  for (const size of fileSizes) {
    // Create a mock file handle with data
    const data = new Uint8Array(size).fill(0);
    const blob = new Blob([data]);
    const file = new File([blob], `test-${size}.arrow`, { type: 'application/octet-stream' });

    const start = performance.now();
    
    try {
      // Read file in chunks
      const chunkSize = 8192;
      let offset = 0;
      while (offset < size) {
        const chunk = await file.slice(offset, offset + chunkSize).arrayBuffer();
        offset += chunkSize;
      }
      
      const end = performance.now();
      const duration = end - start;
      const throughput = (size / duration) * 1000; // bytes per second

      results.push({
        fileSize: size,
        duration,
        throughput,
      });
    } catch (error) {
      console.error(`Error reading file of size ${size}:`, error);
    }
  }

  return results;
}

// Run benchmark
benchmarkFileStreaming().then((results) => {
  console.log('File Streaming Benchmark Results:');
  console.table(results);
});

