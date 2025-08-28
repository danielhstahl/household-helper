import { redirect, type ActionFunctionArgs } from "react-router";
import {
  getToken,
  createUser,
  updateUser,
  deleteUser,
  createSession,
  deleteSession,
} from "./api.tsx";
import { getLoggedInJwt, setLoggedInJwt } from "../state/localState.tsx";
import { getRedirectRoute } from "./routes.tsx";

export const sessionAction = async ({
  request,
  params,
}: ActionFunctionArgs) => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  try {
    switch (request.method) {
      case "POST": {
        const session = await createSession(jwt);
        const redirectRoute = getRedirectRoute(params.agent, session.id);
        return redirect(redirectRoute);
      }
      case "DELETE": {
        const result = await deleteSession(params.sessionId as string, jwt);
        return result;
      }
    }
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};

interface AccessToken {
  access_token: string;
}
export const loginAction = async ({ request }: ActionFunctionArgs) => {
  const formData = await request.formData();
  try {
    const { access_token: accessToken } = (await getToken(
      formData,
    )) as AccessToken;
    setLoggedInJwt(accessToken);
    return redirect("/");
  } catch (error) {
    console.log(error);
    return { error };
  }
};

export const userAction = async ({ request }: ActionFunctionArgs) => {
  const formData = await request.formData();
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  try {
    const { id, username, password, roles } = JSON.parse(
      formData.get("data") as string,
    );
    switch (request.method) {
      case "POST": {
        const postUser = await createUser(username, password, roles, jwt);
        return postUser;
      }
      case "PATCH": {
        const patchUser = await updateUser(id, username, password, roles, jwt);
        return patchUser;
      }
      case "DELETE": {
        const delUser = await deleteUser(id, jwt);
        return delUser;
      }
    }
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};
