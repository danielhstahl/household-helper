import {
  redirect,
  type ActionFunctionArgs,
  type LoaderFunctionArgs,
} from "react-router";
import {
  getToken,
  getSessions,
  getUsers,
  createUser,
  updateUser,
  deleteUser,
  getMessages,
  getUser,
  createSession,
} from "./api";
import { AgentSelectionsEnum } from "../state/selectAgent";
import { ActionEnum, type Action } from "../components/TableX";
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
  //const extraUrl = sessions.length === 0 ? "" : `/${sessionId}`;
  return `/${agent || AgentSelectionsEnum.HELPER}/${sessionId}`;
};
// --- Route Loaders ---
export const loadSession = async ({ params }: LoaderFunctionArgs) => {
  const jwt = getLoggedInJwt();
  console.log(params);

  if (!jwt) {
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    //consider getting a singleton to reduce payload size
    const sessions = await getSessions(jwt);
    const sessionId =
      sessions.length === 0 ? (await createSession(jwt)).id : sessions[0].id;

    console.log(sessions);
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
    // Redirect unauthenticated users to the login page
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
  console.log(jwt);
  if (!jwt) {
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    const user = await getUser(jwt);
    console.log(user);
    return user;
    //return redirect(`/${AgentSelectionsEnum.HELPER}`); //{ jwt, session }; // Pass user data to the route component via useLoaderData
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};

export const sessionAction = async ({ params }: ActionFunctionArgs) => {
  const jwt = getLoggedInJwt();
  console.log(params);
  if (!jwt) {
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    const session = await createSession(jwt);
    console.log(session);
    //redirect to new session
    const redirectRoute = getRedirectRoute(params.agent, session.id);
    return redirect(redirectRoute);
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};

export const logoutAction = () => {
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
    // Return an error object to the component
    return { error };
  }
};

export const setUserAction = async ({ request }: ActionFunctionArgs) => {
  const formData = await request.formData();
  const jwt = getLoggedInJwt();
  const actionData = formData.get("actionData") as string;
  const actionType = formData.get("actionType") as Action;
  console.log(jwt);
  if (!jwt) {
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    const { id, username, password, roles } = JSON.parse(actionData);
    switch (actionType) {
      case ActionEnum.Create:
        return createUser(username, password, roles, jwt);
      case ActionEnum.Update:
        return updateUser(id, username, password, roles, jwt);
      case ActionEnum.Delete:
        return deleteUser(id, username, password, jwt);
    }
    //request.body;
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
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    const users = await getUsers(jwt);
    console.log(users);
    return users;
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};
/*
export const loadMessages = async ({ params }: LoaderFunctionArgs) => {
  //const formData = await request.formData();
  console.log(params);
  const jwt = getLoggedInJwt();

  //const text = formData.get("text") as string;

  if (!jwt) {
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    //console.log(text);
    //need to actually get previous messages
    const response = await Promise.resolve([]); //getMessages("placeholder", jwt);
    return response;
    //request.body;
  } catch (error) {
    console.log(error);
    setLoggedInJwt(null);
    return redirect("/login");
  }
};*/
