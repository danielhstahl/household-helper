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
