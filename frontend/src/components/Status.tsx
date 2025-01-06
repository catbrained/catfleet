import { useEffect, useRef, useState } from "react";

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
  const [error, setError] = useState();
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState<Status | null>(null);

  const abortControllerRef = useRef<AbortController | null>(null);

  useEffect(() => {
    const fetchStatus = async () => {
      abortControllerRef.current?.abort();
      abortControllerRef.current = new AbortController();

      setIsLoading(true);

      try {
        const response = await fetch(`${BASE_URL}/status`, {
          signal: abortControllerRef.current?.signal
        });
        const status = await response.json() as Status;
        setStatus(status);
      } catch (e: any) {
        if (e.name === "AbortError") {
          console.log("Aborted request");
          return;
        }

        setError(e);
      } finally {
        setIsLoading(false);
      }
    };

    fetchStatus();
  }, []);

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div>Something went wrong! Please try again.</div>;
  }

  return (
    <>
      <h1>Status</h1>
      <ul>
        <li>SpaceTraders API: {status?.status}</li>
        <li>SpaceTraders Version: {status?.version}</li>
        <li>Last reset date: {status?.resetDate}</li>
        <li>Next reset date: {status?.serverResets.next}</li>
        <li>Reset frequency: {status?.serverResets.frequency}</li>
      </ul>
    </>
  )
}

export default Status
