import { redirect, type ActionFunctionArgs } from "react-router";
import { getToken, getSession, getUsers } from "./api";

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

//import { getLoggedInUser, logoutUser, loginUser } from "./auth";

// --- Route Loaders ---
export const protectedLoader = async () => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    // Redirect unauthenticated users to the login page
    return redirect("/login");
  }
  try {
    const session = await getSession(jwt);
    console.log(session);
    return { jwt, session }; // Pass user data to the route component via useLoaderData
  } catch (error) {
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
