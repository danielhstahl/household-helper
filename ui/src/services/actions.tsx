import { redirect, type ActionFunctionArgs } from "react-router";
import {
  getToken,
  createUser,
  updateUser,
  deleteUser,
  createSession,
  deleteSession,
  uploadFileToKnowledgeBase,
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

export const uploadFile = async ({ request, params }: ActionFunctionArgs) => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  const formData = await request.formData();
  const kbId = parseInt(params.kbId!);
  try {
    await uploadFileToKnowledgeBase(kbId, formData, jwt);
  } catch (error) {
    console.log(error);
    return { error };
  }
};

export const userAction = async ({ request }: ActionFunctionArgs) => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  const formData = await request.formData();
  try {
    switch (request.method) {
      case "POST": {
        const { username, password, roles } = JSON.parse(
          formData.get("data") as string,
        );
        const postUser = await createUser(username, password, roles, jwt);
        return postUser;
      }
      case "PATCH": {
        const { id, username, password, roles } = JSON.parse(
          formData.get("data") as string,
        );
        const patchUser = await updateUser(id, username, password, roles, jwt);
        return patchUser;
      }
      case "DELETE": {
        const { id } = JSON.parse(formData.get("data") as string);
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
