import Table, { type Action, ActionEnum } from "../components/TableX";
import { useLoaderData, useFetcher } from "react-router";
import Grid from "@mui/material/Grid";
interface User {
  id: number;
  username: string;
  roles: string[];
}
const mapActionToRequest = (actionType: Action) => {
  switch (actionType) {
    case ActionEnum.Create:
      return "POST";
    case ActionEnum.Update:
      return "PATCH";
    case ActionEnum.Delete:
      return "DELETE";
  }
};
const Users = () => {
  const fetcher = useFetcher();
  const users = useLoaderData() as User[];

  const onChange = (
    type: Action,
    id: string | number,
    username: string,
    password: string | undefined,
    roles: string[],
  ) => {
    const formData = new FormData();
    formData.append("data", JSON.stringify({ id, username, password, roles }));
    fetcher.submit(formData, { method: mapActionToRequest(type) });
  };
  return (
    <Grid size={{ xs: 12 }}>
      <Table users={users} onChange={onChange} />
    </Grid>
  );
};

export default Users;
