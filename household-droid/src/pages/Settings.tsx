import { useLoaderData, useSubmit } from "react-router";
import { setUserAction } from "../services/auth";
import Table from "../components/TableX";

interface User {
  id: number;
  username: string;
  roles: string[];
}

const Settings = () => {
  const users = useLoaderData() as User[];
  //const usersWithId = users.map((user, index) => ({ ...user, id: index }));
  const submit = useSubmit();

  const onChange = (
    type: string,
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
    submit(formData, { method: "post" });
  };
  return <Table users={users} onChange={onChange} />;
};
export default Settings;
