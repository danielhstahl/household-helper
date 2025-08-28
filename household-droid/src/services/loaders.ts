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
  getMessages,
  getUser,
  createSession,
} from "./api";
import { getLoggedInJwt, setLoggedInJwt } from "../state/localState.ts";
import { getRedirectRoute } from "./routes.ts";
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
