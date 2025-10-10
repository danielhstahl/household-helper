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
import Settings from "./pages/Settings.tsx";
import Metrics from "./components/Metrics.tsx";
import { createBrowserRouter, redirect, RouterProvider } from "react-router";
import {
  logoutLoader,
  loadUsers,
  loadSessionsAndMessages,
  loadUser,
  loadSession,
  loadMetrics,
  loadKnowledgeBase,
} from "./services/loaders.tsx";

import {
  sessionAction,
  userAction,
  loginAction,
  uploadFile,
} from "./services/actions.tsx";
import Users from "./components/Users.tsx";
import KnowledgeBaseUpload from "./components/KnowledgeBase.tsx";

const router = createBrowserRouter([
  {
    path: "/",
    Component: App,
    children: [
      {
        path: "/",
        Component: MainPage, //careful, this has appBar, which also requires agent and session (and will be undefined on first load)
        loader: loadUser,
        children: [
          {
            path: "/",
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
            children: [
              {
                loader: () => redirect("users"),
                index: true,
              },
              {
                path: "users",
                loader: loadUsers,
                action: userAction,
                Component: Users,
              },
              {
                path: "metrics",
                loader: loadMetrics,
                Component: Metrics,
              },
              {
                path: "knowledgebase",
                loader: loadKnowledgeBase,
                Component: KnowledgeBaseUpload,
              },
              {
                path: "knowledgebase/:kbId",
                action: uploadFile,
              },
            ],
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
        loader: logoutLoader,
      },
    ],
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />,
  </StrictMode>,
);
