const apiUrlPrefix = import.meta.env.VITE_TRIP_ATLAS_API_URL ?? "./api";
if (import.meta.env.VITE_TRIP_ATLAS_API_URL) {
  console.log("Using API at:", apiUrlPrefix);
}

export interface Config {
  allow_shutdown_from_frontend: boolean;
}

let config: Config | null = null;

export async function getConfig(): Promise<Config> {
  if (config === null) {
    const response = await fetch(getApiUrl("/config"));
    config = (await response.json()) as Config;
  }
  return config;
}

export function getApiUrl(path: string) {
  return `${apiUrlPrefix}${path}`;
}
