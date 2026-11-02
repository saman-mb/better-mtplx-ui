// Temporary harness: mocks the Tauri IPC bridge so the UI can be rendered and
// screenshotted in a plain browser. Not part of the shipped app.
(function () {
  // Headless Chrome freezes CSS transitions, which paints stale frames.
  const kill = document.createElement("style");
  kill.textContent = "*, *::before, *::after { transition: none !important; }";
  document.documentElement.appendChild(kill);

  const GB = 1024 ** 3;

  const packs = [
    {
      repo_id: "Youssofal/Qwen3.8-27B-MTPLX-Optimized-Quality",
      path: "/Users/saman/.mtplx/models/Youssofal--Qwen3.8-27B-MTPLX-Optimized-Quality",
      size_gb: 29.97,
      is_primary: true,
      has_runtime_contract: true,
      recommended_profile: "turbo",
      validation: { contract_present: true, missing_files: [] },
      delete_command: null,
    },
    {
      repo_id: "Youssofal/Qwen3.8-Flash-Next-MTPLX-Optimized-Speed",
      path: "/Users/saman/.mtplx/models/Youssofal--Qwen3.8-Flash-Next-MTPLX-Optimized-Speed",
      size_gb: 107.2,
      is_primary: true,
      has_runtime_contract: true,
      recommended_profile: "turbo",
      validation: { contract_present: true, missing_files: [] },
      delete_command: null,
    },
    {
      repo_id: "Foresee/Qwen3.8-9B-heretic-uncensored-4bit-MTPLX",
      path: "/Users/saman/.mtplx/models/Foresee--Qwen3.8-9B-heretic-uncensored-4bit-MTPLX",
      size_gb: 5.6,
      is_primary: false,
      has_runtime_contract: false,
      recommended_profile: null,
      validation: { contract_present: false, missing_files: ["mtp.safetensors"] },
      delete_command: null,
    },
  ];

  const catalog = [
    {
      repo: "Youssofal/Qwen3.8-27B-MTPLX-Optimized-Speed",
      owner: "Youssofal",
      branded_name: "Qwen3.8-27B-MTPLX-Optimized-Speed",
      downloads: 26610,
      license: "apache-2.0",
    },
    {
      repo: "Foresee/Qwen3.8-9B-heretic-uncensored-4bit-MTPLX",
      owner: "Foresee",
      branded_name: "Qwen3.8-9B-heretic-uncensored-4bit-MTPLX",
      downloads: 10252,
      license: "apache-2.0",
    },
    {
      repo: "Youssofal/Qwen3.8-Flash-Next-MTPLX-Optimized-Quality",
      owner: "Youssofal",
      branded_name: "Qwen3.8-Flash-Next-MTPLX-Optimized-Quality",
      downloads: 8831,
      license: "other",
    },
  ];

  const tick = {
    instances: [
      {
        id: "inst-8002-1",
        model: "Youssofal/Qwen3.8-27B-MTPLX-Optimized-Quality",
        port: 8002,
        status: "ready",
        memoryBytes: 28.8 * GB,
        weightsBytes: 27.9 * GB,
        tokensPerSecond: 84.31,
        error: null,
        logs: [],
        startedAt: 1730000000,
      },
      {
        id: "inst-8003-2",
        model: "Youssofal/Qwen3.8-Flash-Next-MTPLX-Optimized-Speed",
        port: 8003,
        status: "loading",
        memoryBytes: 29.8 * GB,
        weightsBytes: 77.3 * GB,
        tokensPerSecond: null,
        error: null,
        logs: ["[5/6] Model load in progress"],
        startedAt: 1730000060,
      },
    ],
    memory: {
      totalBytes: 256 * GB,
      availableBytes: 168 * GB,
      usedBytes: 88 * GB,
      usedRatio: 88 / 256,
    },
    external: [
      {
        port: 8000,
        modelId: "mtplx-qwen38-27b-optimized-quality",
        contextLength: 262144,
        memoryBytes: 28.8 * GB,
        weightsBytes: 27.9 * GB,
      },
      {
        port: 8001,
        modelId: "mtplx-flash-next-optimized-speed",
        contextLength: 262144,
        memoryBytes: 29.8 * GB,
        weightsBytes: 77.3 * GB,
      },
    ],
  };

  const listeners = {};

  window.__TAURI_INTERNALS__ = {
    transformCallback(callback) {
      const id = Math.floor(Math.random() * 1e9);
      window[`_${id}`] = callback;
      return id;
    },
    unregisterCallback(id) {
      delete window[`_${id}`];
    },
    async invoke(cmd, args) {
      switch (cmd) {
        case "plugin:event|listen":
          listeners[args.event] = window[`_${args.handler}`];
          return 1;
        case "plugin:event|unlisten":
          return null;
        case "get_settings":
          return {
            defaultPort: 8000,
            defaultProfile: "turbo",
            contextWindow: null,
            host: "127.0.0.1",
            apiKey: null,
            schedulerMode: "serial",
            extraArgs: [],
            theme: "dark",
          };
        case "list_models":
          return { cache_dir: "/Users/saman/.mtplx/models", models: packs };
        case "discover_models":
          return catalog;
        case "list_instances":
          return tick.instances;
        case "memory_snapshot":
          return tick.memory;
        case "suggest_port":
          return 8002;
        case "mtplx_status":
          return {};
        default:
          return null;
      }
    },
  };

  setInterval(() => {
    const handler = listeners["instances://tick"];
    if (handler) handler({ event: "instances://tick", id: 1, payload: tick });
  }, 1000);
})();

  // Allow the harness to pick a view via the URL hash.
  const VIEWS = { instances: 0, library: 1, settings: 2 };
  const wanted = VIEWS[location.hash.replace("#", "")];
  if (wanted !== undefined) {
    setTimeout(() => {
      const buttons = document.querySelectorAll("nav button");
      if (buttons[wanted]) buttons[wanted].click();
    }, 400);
  }
