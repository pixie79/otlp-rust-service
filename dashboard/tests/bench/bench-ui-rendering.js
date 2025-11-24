/**
 * Performance benchmark for UI rendering
 * Measures rendering performance for different component sizes
 */

async function benchmarkUIRendering() {
  const results = [];

  const traceCounts = [10, 100, 1000, 10000];
  const metricPointCounts = [100, 1000, 10000, 100000];

  // Benchmark trace list rendering
  for (const traceCount of traceCounts) {
    const container = document.createElement('div');
    container.style.display = 'none';
    document.body.appendChild(container);

    const start = performance.now();
    
    // Simulate rendering traces
    for (let i = 0; i < traceCount; i++) {
      const traceEl = document.createElement('div');
      traceEl.textContent = `Trace ${i}`;
      container.appendChild(traceEl);
    }
    
    const end = performance.now();
    const duration = end - start;

    results.push({
      component: 'TraceList',
      itemCount: traceCount,
      duration,
      itemsPerSecond: (traceCount / duration) * 1000,
    });

    document.body.removeChild(container);
  }

  // Benchmark metric graph rendering
  for (const pointCount of metricPointCounts) {
    const container = document.createElement('div');
    container.style.display = 'none';
    document.body.appendChild(container);

    const start = performance.now();
    
    // Simulate rendering graph points
    // In real implementation, this would use Plotly.js
    const data = Array.from({ length: pointCount }, (_, i) => ({
      x: i,
      y: Math.random() * 100,
    }));
    
    const end = performance.now();
    const duration = end - start;

    results.push({
      component: 'MetricGraph',
      pointCount,
      duration,
      pointsPerSecond: (pointCount / duration) * 1000,
    });

    document.body.removeChild(container);
  }

  return results;
}

// Run benchmark when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    benchmarkUIRendering().then((results) => {
      console.log('UI Rendering Benchmark Results:');
      console.table(results);
    });
  });
} else {
  benchmarkUIRendering().then((results) => {
    console.log('UI Rendering Benchmark Results:');
    console.table(results);
  });
}

