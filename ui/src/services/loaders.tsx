import { redirect, type LoaderFunctionArgs } from "react-router";
import {
  getSessions,
  getMostRecentSession,
  getUsers,
  getMessages,
  getUser,
  createSession,
} from "./api.tsx";
import { getLoggedInJwt, setLoggedInJwt } from "../state/localState.tsx";
import { getRedirectRoute } from "./routes.tsx";
import { type Message } from "../components/Output.tsx";

// --- Route Loaders ---
export const loadSession = async ({ params }: LoaderFunctionArgs) => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    return redirect("/login");
  }
  try {
    const session = await getMostRecentSession(jwt);
    const sessionId = session ? session.id : (await createSession(jwt)).id;
    //redirect to route that loads loadSessionsAndMessages
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
    const { sessionId } = params;
    const [sessions, messages] = await Promise.all([
      getSessions(jwt),
      getMessages(sessionId as string, jwt).then((messages) => {
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

export const logoutLoader = () => {
  setLoggedInJwt(null);
  return redirect("/login");
};

export const loadUsers = async () => {
  const jwt = getLoggedInJwt();
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
