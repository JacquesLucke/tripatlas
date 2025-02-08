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
    let response = await fetch(url, { method: "POST" });
    if (!response.ok) {
      alert("Failed to shutdown server.");
      return;
    }
    window.close();
  }

  if (config?.allow_shutdown_from_frontend) {
    return (
      <button
        className="hover:border-black border-2 border-transparent duration-200 p-1 rounded m-2"
        onClick={shutdown}
      >
        Shutdown
      </button>
    );
  }

  return <div>Trip Atlas</div>;
}

export default App;
