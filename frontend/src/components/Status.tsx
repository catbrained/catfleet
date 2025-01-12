import { useQueryClient } from "@tanstack/react-query";
import { $api } from "../api.ts";

function Status() {
  const queryClient = useQueryClient();
  const { isPending, isError, isFetching, data, error } = $api.useQuery(
    "get",
    "/status",
    {},
    {
      staleTime: 1000 * 10,
      refetchInterval: 1000 * 15,
    },
    queryClient,
  );

  if (isPending) {
    return <span>Loading...</span>;
  }

  if (isError) {
    return (
      <span>
        Something went wrong! Please try again. (Error message: {error})
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
