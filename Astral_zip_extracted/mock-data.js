// Mock SystemMetrics generator — emulates the SSE feed Astral's backend sends.
// Exposed via window.AstralMock so JSX scripts can import it.

(function () {
  function rand(min, max) { return min + Math.random() * (max - min); }
  function clamp(v, lo, hi) { return Math.max(lo, Math.min(hi, v)); }

  // Smooth-walking series with mean-reversion
  function makeWalk(start, mean, vol, lo, hi) {
    let v = start;
    return function step() {
      v += (mean - v) * 0.04 + (Math.random() - 0.5) * vol;
      v = clamp(v, lo, hi);
      return v;
    };
  }

  function gen(prev) {
    const cpu = (prev && prev._cpuStep) || makeWalk(38, 42, 14, 4, 99);
    const memUsed = (prev && prev._memStep) || makeWalk(7.3, 7.6, 0.35, 5.1, 13.8);
    const tx = (prev && prev._txStep) || makeWalk(96 * 1024, 110 * 1024, 60 * 1024, 4 * 1024, 9 * 1024 * 1024);
    const rx = (prev && prev._rxStep) || makeWalk(420 * 1024, 480 * 1024, 240 * 1024, 8 * 1024, 24 * 1024 * 1024);
    const dRead = (prev && prev._drStep) || makeWalk(1.4 * 1024 * 1024, 1.8 * 1024 * 1024, 0.6 * 1024 * 1024, 0, 90 * 1024 * 1024);
    const dWrite = (prev && prev._dwStep) || makeWalk(0.6 * 1024 * 1024, 0.9 * 1024 * 1024, 0.4 * 1024 * 1024, 0, 60 * 1024 * 1024);

    const totalMemory = 15.5 * 1024 * 1024 * 1024;
    const cpuVal = cpu();
    const used = memUsed() * 1024 * 1024 * 1024;

    return {
      hostname: "edge-fra-01",
      os_name: "Debian GNU/Linux",
      os_version: "12",
      uptime: 60 * 60 * 24 * 18 + 60 * 60 * 7 + 60 * 42, // ~18d 7h 42m
      cpu_usage: cpuVal,
      cpu_cores: 12,
      cpu_load: [cpuVal * 0.018, cpuVal * 0.015, cpuVal * 0.012], // 1/5/15m load avg approx
      total_memory: totalMemory,
      used_memory: used,
      total_swap: 4 * 1024 * 1024 * 1024,
      used_swap: 0.18 * 1024 * 1024 * 1024,
      network_tx: tx(),
      network_rx: rx(),
      disks: [
        { name: "nvme0n1", mount_point: "/", total_space: 512 * 1024 * 1024 * 1024, available_space: 406 * 1024 * 1024 * 1024, read_bytes: dRead(), written_bytes: dWrite() },
        { name: "nvme0n2", mount_point: "/var", total_space: 256 * 1024 * 1024 * 1024, available_space: 188 * 1024 * 1024 * 1024, read_bytes: 0, written_bytes: 0 },
        { name: "sda1",    mount_point: "/data", total_space: 2000 * 1024 * 1024 * 1024, available_space: 1240 * 1024 * 1024 * 1024, read_bytes: 0, written_bytes: 0 }
      ],
      processes: makeProcesses(cpuVal),
      _cpuStep: cpu, _memStep: memUsed, _txStep: tx, _rxStep: rx, _drStep: dRead, _dwStep: dWrite,
    };
  }

  function makeProcesses(cpu) {
    // Stable PIDs; vary CPU/mem a little so the list feels alive.
    const base = [
      { pid: 14882, name: "astral",          cpu: cpu * 0.04 + rand(0.3, 1.4),  mem:   9.4 * 1024 * 1024 },
      { pid:  1834, name: "postgres",        cpu: cpu * 0.18 + rand(1.0, 3.6),  mem: 412.0 * 1024 * 1024 },
      { pid:  2891, name: "node",            cpu: cpu * 0.22 + rand(0.6, 4.8),  mem: 287.2 * 1024 * 1024 },
      { pid:  1024, name: "dockerd",         cpu: rand(0.0, 0.8),               mem: 154.2 * 1024 * 1024 },
      { pid:  1029, name: "containerd",      cpu: rand(0.0, 0.4),               mem:  44.1 * 1024 * 1024 },
      { pid: 19204, name: "nginx",           cpu: cpu * 0.05 + rand(0.0, 0.6),  mem:  18.4 * 1024 * 1024 },
      { pid:  4012, name: "redis-server",    cpu: rand(0.0, 0.9),               mem:  62.7 * 1024 * 1024 },
      { pid: 30182, name: "tokio-rt-worker", cpu: rand(0.0, 0.4),               mem:   9.4 * 1024 * 1024 },
      { pid:  2102, name: "systemd-journal", cpu: rand(0.0, 0.3),               mem:  21.4 * 1024 * 1024 },
      { pid:     1, name: "init",            cpu: 0.0,                          mem:   8.2 * 1024 * 1024 },
      { pid:  3401, name: "lifecycle-serve", cpu: rand(0.0, 0.2),               mem:  38.3 * 1024 * 1024 },
      { pid:  6722, name: "node",            cpu: rand(0.0, 1.0),               mem:  92.1 * 1024 * 1024 },
    ];
    return base.map(p => ({ pid: p.pid, name: p.name, cpu_usage: p.cpu, memory: p.mem }));
  }

  // Build pre-rolled history for the chart screens (6h / 24h / 7d)
  function buildHistory(points, jitter) {
    const out = [];
    const now = Date.now();
    const span = points;
    for (let i = 0; i < span; i++) {
      const t = now - (span - i) * (3600 * 1000 / (points / Math.max(1, points / 200)));
      // Daily sine + small noise
      const x = i / span * Math.PI * 6;
      const cpu_usage = clamp(38 + Math.sin(x) * 18 + Math.sin(x * 3.1) * 8 + (Math.random() - 0.5) * jitter, 4, 98);
      const used_memory = clamp(7.3 + Math.sin(x * 1.4) * 0.6 + (Math.random() - 0.5) * 0.3, 5.5, 12.5) * 1024 * 1024 * 1024;
      const network_tx = clamp(110 * 1024 + Math.sin(x * 2.2) * 80 * 1024 + (Math.random() - 0.4) * 400 * 1024, 0, 9 * 1024 * 1024);
      const network_rx = clamp(480 * 1024 + Math.sin(x * 1.7) * 260 * 1024 + (Math.random() - 0.4) * 800 * 1024, 0, 24 * 1024 * 1024);
      const disk_read_rate = clamp(1.4 * 1024 * 1024 + (Math.random() - 0.5) * 2 * 1024 * 1024, 0, 60 * 1024 * 1024);
      const disk_write_rate = clamp(0.9 * 1024 * 1024 + (Math.random() - 0.5) * 1.4 * 1024 * 1024, 0, 30 * 1024 * 1024);
      out.push({ timestamp: t, cpu_usage, used_memory, network_tx, network_rx, disk_read_rate, disk_write_rate });
    }
    return out;
  }

  // Sample alert history
  function buildAlerts() {
    const now = Date.now();
    return [
      { kind: "cpu",  message: "CPU usage sustained above 90% for 5m",   value: 94.2, threshold: 90, timestamp: now - 1000 * 60 * 22 },
      { kind: "ram",  message: "Memory usage sustained above 90% for 5m", value: 91.7, threshold: 90, timestamp: now - 1000 * 60 * 60 * 3.4 },
      { kind: "cpu",  message: "CPU usage sustained above 90% for 5m",   value: 96.4, threshold: 90, timestamp: now - 1000 * 60 * 60 * 18 },
      { kind: "ram",  message: "Memory usage sustained above 90% for 5m", value: 92.0, threshold: 90, timestamp: now - 1000 * 60 * 60 * 27 },
      { kind: "cpu",  message: "CPU usage sustained above 90% for 5m",   value: 93.1, threshold: 90, timestamp: now - 1000 * 60 * 60 * 49 },
      { kind: "cpu",  message: "CPU usage sustained above 90% for 5m",   value: 91.4, threshold: 90, timestamp: now - 1000 * 60 * 60 * 72 },
    ];
  }

  window.AstralMock = { gen, buildHistory, buildAlerts };
})();
