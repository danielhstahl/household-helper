import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";
import App from "./App.tsx";
import MainChat from "./pages/MainChat.tsx";
import Auth from "./pages/Auth.tsx";
import { createBrowserRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import {
  getLoggedInJwt,
  loginAction,
  logoutAction,
  protectedLoader,
} from "./services/auth.ts";

const router = createBrowserRouter([
  {
    path: "/",
    Component: App,
    loader() {
      // Root loader can fetch global data or simply pass user
      return { user: getLoggedInJwt() };
    },
    children: [
      {
        index: true,
        Component: MainChat,
        loader: protectedLoader, // Prevent un-authenticated from seeing main page
        action: loginAction, // Handles login form submission
      },
      //{ path: "settings", Component: Settings },
      {
        path: "auth",
        Component: Auth,
      },
      {
        path: "logout",
        action: logoutAction,
      },
    ],
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />,
  </StrictMode>,
);
