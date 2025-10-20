import Table, { type Action, ActionEnum } from "../components/TableX";
import { useLoaderData, useFetcher } from "react-router";
import Grid from "@mui/material/Grid";
import { type UserResponse } from "../services/models";

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
  const fetcher = useFetcher<typeof useLoaderData>();
  const users = useLoaderData<UserResponse[]>();

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
