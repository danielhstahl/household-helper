import { useLoaderData, useFetcher } from "react-router";
import Table, { type Action, ActionEnum } from "../components/TableX";

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

const Settings = () => {
  const users = useLoaderData() as User[];
  const fetcher = useFetcher();
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
  return <Table users={users} onChange={onChange} />;
};
export default Settings;
