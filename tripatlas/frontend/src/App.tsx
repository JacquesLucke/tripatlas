import {
  QueryClient,
  QueryClientProvider,
  useQuery,
} from "@tanstack/react-query";
import { Config, getApiUrl, getConfig } from "./api";

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Settings />
    </QueryClientProvider>
  );
}

function Settings() {
  const { data: config } = useQuery<Config>({
    queryKey: ["config"],
    queryFn: getConfig,
  });

  async function shutdown() {
    const url = getApiUrl("/shutdown");
    await fetch(url, { method: "POST" });
    window.close();
  }

  if (config?.allow_shutdown_from_frontend) {
    return (
      <button
        className="hover:bg-amber-200 p-2 rounded bg-amber-100 m-4"
        onClick={shutdown}
      >
        Shutdown
      </button>
    );
  }

  return <div className="text-red-500">Hello</div>;
}

export default App;
