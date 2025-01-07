import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import Status from "./components/Status"

const queryClient = new QueryClient();

function App() {

  return (
    <QueryClientProvider client={queryClient}>
      <Status />
    </QueryClientProvider>
  )
}

export default App
