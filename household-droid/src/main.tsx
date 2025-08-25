import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";
import App from "./App.tsx";
import MainPage from "./pages/MainPage.tsx";
import MainChat from "./pages/MainChat.tsx";
import Login from "./pages/Login.tsx";
import Settings from "./pages/Settings";
import { createBrowserRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import {
  loginAction,
  logoutAction,
  setUserAction,
  loadUsers,
  loadSessionsAndMessages,
  sessionAction,
  loadUser,
  loadSession,
} from "./services/auth.ts";

const router = createBrowserRouter([
  {
    path: "/",
    Component: App,
    children: [
      {
        path: "/",
        Component: MainPage, //careful, I think this has appBar, which also requires agent and session (and will be undefined on first load)
        loader: loadUser,
        children: [
          {
            path: "/",
            id: "sessionLoader",
            children: [
              {
                path: "/",
                loader: loadSession, //redirects to :agent/:sessionId
              },
              {
                path: ":agent/:sessionId",
                Component: MainChat,
                loader: loadSessionsAndMessages, // messages for the session...do I also really want sessions here?  I could put it up a level
                action: sessionAction, // create new session or delete session
              },
            ],
          },
          {
            path: "settings",
            Component: Settings,
            loader: loadUsers,
            action: setUserAction,
          },
        ],
      },
      {
        path: "login",
        Component: Login,
        action: loginAction, // Handles login form submission
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
