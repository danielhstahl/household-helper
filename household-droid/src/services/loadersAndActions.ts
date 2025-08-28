import {
  redirect,
  type ActionFunctionArgs,
  type LoaderFunctionArgs,
} from "react-router";
import {
  getToken,
  getSessions,
  getMostRecentSession,
  getUsers,
  createUser,
  updateUser,
  deleteUser,
  getMessages,
  getUser,
  createSession,
  deleteSession,
} from "./api";
import { AgentSelectionsEnum } from "../state/selectAgent";
import { type Message } from "../components/Output";

const USER_JWT_KEY = "user-jwt";

export const getLoggedInJwt = () => {
  const jwt = localStorage.getItem(USER_JWT_KEY);
  return jwt || null;
};

export const setLoggedInJwt = (jwt: string | null) => {
  if (jwt) {
    localStorage.setItem(USER_JWT_KEY, jwt);
  } else {
    localStorage.removeItem(USER_JWT_KEY);
  }
};

//exported for testing
export const getRedirectRoute = (
  agent: string | undefined,
  sessionId: string,
) => {
  //default to session that was most recently started
  return `/${agent || AgentSelectionsEnum.HELPER}/${sessionId}`;
};
// --- Route Loaders ---
export const loadSession = async ({ params }: LoaderFunctionArgs) => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  try {
    const session = await getMostRecentSession(jwt);
    const sessionId = session ? session.id : (await createSession(jwt)).id;
    const redirectRoute = getRedirectRoute(params.agent, sessionId);
    return redirect(redirectRoute);
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};

//combine since need both in single component
export const loadSessionsAndMessages = async ({
  params,
}: LoaderFunctionArgs) => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  try {
    const [sessions, messages] = await Promise.all([
      getSessions(jwt),
      getMessages(params.sessionId as string, jwt).then((v) => {
        const messages = v.messages;
        messages.sort((a: Message, b: Message) =>
          a.timestamp < b.timestamp ? -1 : 1,
        );
        return messages;
      }),
    ]);
    return { sessions, messages };
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};

export const loadUser = async () => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  try {
    const user = await getUser(jwt);
    return user;
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};

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

export const logoutLoader = () => {
  setLoggedInJwt(null);
  return redirect("/login");
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

export const loadUsers = async () => {
  const jwt = getLoggedInJwt();
  console.log(jwt);
  if (!jwt) {
    return redirect("/login");
  }
  try {
    const users = await getUsers(jwt);
    return users;
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};
