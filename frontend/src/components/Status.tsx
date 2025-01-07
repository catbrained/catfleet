import { useQuery } from "@tanstack/react-query";

const BASE_URL = "http://localhost:5173";

interface Status {
  status: string;
  version: string;
  resetDate: string;
  description: string;
  stats: Stats;
  leaderboards: Leaderboards;
  serverResets: ServerResets;
  announcements: Announcement[];
  links: Link[];
}

interface Link {
  name: string;
  url: string;
}

interface Announcement {
  title: string;
  body: string;
}

interface ServerResets {
  next: string;
  frequency: string;
}

interface Stats {
  agents: number;
  ships: number;
  systems: number;
  waypoints: number;
}

interface Leaderboards {
  mostCredits: AgentCredits[];
  mostSubmittedCharts: AgentCharts[];
}

interface AgentCredits {
  agentSymbol: string;
  credits: number;
}

interface AgentCharts {
  agentSymbol: string;
  chartCount: number;
}

function Status() {
  const fetchStatus = async (): Promise<Status> => {
    const response = await fetch(`${BASE_URL}/status`);
    if (!response.ok) {
      throw new Error("Network response was not ok");
    }
    return response.json();
  };

  const { isPending, isError, isFetching, data, error } = useQuery({
    queryKey: ["status"],
    queryFn: fetchStatus,
    staleTime: 1000 * 10,
    refetchInterval: 1000 * 15,
  });

  if (isPending) {
    return <span>Loading...</span>;
  }

  if (isError) {
    return (
      <span>
        Something went wrong! Please try again. (Error message: {error.message})
      </span>
    );
  }

  return (
    <>
      <h1>Status</h1>
      <ul>
        <li>SpaceTraders API: {data.status}</li>
        <li>SpaceTraders Version: {data.version}</li>
        <li>Last reset date: {data.resetDate}</li>
        <li>Next reset date: {data.serverResets.next}</li>
        <li>Reset frequency: {data.serverResets.frequency}</li>
      </ul>
      {isFetching && <span>Refreshing...</span>}
    </>
  );
}

export default Status;
