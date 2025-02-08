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
      <div className="absolute top-0 right-0 bg-sky-100 p-2 rounded-bl-md shadow-md">
        <Settings />
      </div>
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

  return <div>Trip Atlas</div>;
}

export default App;
