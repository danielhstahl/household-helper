import { redirect, type ActionFunctionArgs } from "react-router";
import { getToken } from "./api";

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
export const protectedLoader = () => {
  const jwt = getLoggedInJwt();
  if (!jwt) {
    // Redirect unauthenticated users to the login page
    // You can also add a `?message=login_required` to the URL for context
    //const params = new URLSearchParams();
    //params.set("from", new URL(request.url).pathname);
    return redirect("/login");
  }
  return jwt; // Pass user data to the route component via useLoaderData
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
  //const username = formData.get("username") as string;
  // const password = formData.get("password") as string;

  try {
    const { access_token: accessToken } = (await getToken(
      formData,
    )) as AccessToken;
    // Redirect to the 'from' path or dashboard on successful login
    return accessToken;
  } catch (error) {
    // Return an error object to the component
    return { error };
  }
};
