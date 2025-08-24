import { useLoaderData, useFetcher } from "react-router";
import { type Action } from "../components/TableX";
import Table from "../components/TableX";

interface User {
  id: number;
  username: string;
  roles: string[];
}

const Settings = () => {
  const users = useLoaderData() as User[];
  const fetcher = useFetcher();
  console.log(users);
  const onChange = (
    type: Action,
    id: string | number,
    username: string,
    password: string | undefined,
    roles: string[],
  ) => {
    const formData = new FormData();

    formData.append(
      "actionData",
      JSON.stringify({ id, username, password, roles }),
    );
    formData.append("actionType", type);
    fetcher.submit(formData, { method: "post" });
  };
  return <Table users={users} onChange={onChange} />;
};
export default Settings;
