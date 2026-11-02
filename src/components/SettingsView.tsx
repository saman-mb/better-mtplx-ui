import { useEffect, useState } from "react";
import { getSettings, setSettings } from "../lib/ipc";
import type { Settings } from "../lib/types";

const PROFILES = [
  "turbo",
  "stable",
  "performance-cold",
  "sustained",
  "exact",
  "max-diagnostic",
];

const SCHEDULERS = ["serial", "cooperative", "ar_batch", "mtp_batch", "hyper"];

/**
 * Settings view.
 *
 * Deliberately small: the values that change how a new instance is launched.
 * Anything deeper belongs in `mtplx serve` flags, which the raw-arguments
 * field exposes.
 */
export function SettingsView() {
  const [settings, setSettingsState] = useState<Settings | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getSettings()
      .then(setSettingsState)
      .catch(() => setSettingsState(null));
  }, []);

  if (!settings) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-xs text-slate-500">Loading settings…</p>
      </div>
    );
  }

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setSettingsState({ ...settings, [key]: value });
    setSaved(false);
  };

  const save = async () => {
    await setSettings(settings);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="h-full overflow-y-auto px-6 py-5">
      <div className="mb-5 flex items-center gap-3">
        <h1 className="text-sm font-semibold text-slate-100">Settings</h1>
        <button
          onClick={() => void save()}
          className="rounded-md bg-accent-500 px-3 py-1.5 text-xs font-semibold text-ink-950 transition-colors hover:bg-accent-400"
        >
          {saved ? "Saved" : "Save"}
        </button>
      </div>

      <div className="max-w-xl space-y-5">
        <Field
          label="Default port"
          hint="First port assigned to a new instance."
        >
          <input
            type="number"
            value={settings.defaultPort}
            onChange={(event) =>
              update("defaultPort", Number(event.target.value) || 8000)
            }
            className={inputClass}
          />
        </Field>

        <Field
          label="Default profile"
          hint="MTPLX serving profile for new instances."
        >
          <select
            value={settings.defaultProfile}
            onChange={(event) => update("defaultProfile", event.target.value)}
            className={inputClass}
          >
            {PROFILES.map((profile) => (
              <option key={profile} value={profile}>
                {profile}
              </option>
            ))}
          </select>
        </Field>

        <Field
          label="Scheduler mode"
          hint="How the engine batches concurrent requests."
        >
          <select
            value={settings.schedulerMode}
            onChange={(event) => update("schedulerMode", event.target.value)}
            className={inputClass}
          >
            {SCHEDULERS.map((mode) => (
              <option key={mode} value={mode}>
                {mode}
              </option>
            ))}
          </select>
        </Field>

        <Field
          label="Context window"
          hint="Token budget per request. Leave empty to use the profile default."
        >
          <input
            type="number"
            value={settings.contextWindow ?? ""}
            placeholder="profile default"
            onChange={(event) =>
              update(
                "contextWindow",
                event.target.value ? Number(event.target.value) : null,
              )
            }
            className={inputClass}
          />
        </Field>

        <Field
          label="Bind host"
          hint="127.0.0.1 keeps the endpoint on this Mac only."
        >
          <input
            value={settings.host}
            onChange={(event) => update("host", event.target.value)}
            className={inputClass}
          />
        </Field>

        <Field
          label="API key"
          hint="Optional. Sent to the local endpoint when set."
        >
          <input
            type="password"
            value={settings.apiKey ?? ""}
            placeholder="none"
            onChange={(event) => update("apiKey", event.target.value || null)}
            className={inputClass}
          />
        </Field>

        <Field
          label="Extra serve arguments"
          hint="Raw flags appended to every `mtplx serve` call, space separated."
        >
          <input
            value={settings.extraArgs.join(" ")}
            placeholder="--reasoning-effort high"
            onChange={(event) =>
              update(
                "extraArgs",
                event.target.value.split(/\s+/).filter(Boolean),
              )
            }
            className={inputClass}
          />
        </Field>
      </div>
    </div>
  );
}

const inputClass =
  "w-full rounded-lg border border-ink-700 bg-ink-850 px-3 py-2 text-xs text-slate-200 outline-none placeholder:text-slate-500 focus:border-accent-500";

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <label className="block text-xs font-medium text-slate-200">
        {label}
      </label>
      <p className="mb-2 mt-0.5 text-[11px] text-slate-500">{hint}</p>
      {children}
    </div>
  );
}
