import type { AgentSelections } from "../state/selectAgent";
import type {
  QueryLatency,
  QueryTools,
  Token,
  UserResponse,
  Message,
  SessionDB,
  KnowledgeBase,
  RoleType,
} from "./models";

export interface StatusResponse {
  status: string;
}

export interface ChatToken {
  tokenType: string;
  tokens: string;
}
const getHeaders = (jwt: string) => ({
  "Content-Type": "application/json",
  Authorization: `Bearer ${jwt}`,
});

async function fetchWithAuth<T>(
  url: string,
  jwt: string,
  options: RequestInit = {},
): Promise<T> {
  const response = await fetch(url, {
    ...options,
    headers: {
      ...getHeaders(jwt),
      ...options.headers,
    },
  });
  if (response.ok) {
    return response.json();
  }
  throw new Error(await response.text());
}

export async function getSessions(jwt: string): Promise<SessionDB[]> {
  return fetchWithAuth<SessionDB[]>("/api/session", jwt);
}

export async function getMostRecentSession(
  jwt: string,
): Promise<SessionDB | undefined> {
  const response = await fetch("/api/session/recent", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    return response.json();
  }
  return;
}

export async function createSession(jwt: string): Promise<SessionDB> {
  return fetchWithAuth<SessionDB>("/api/session", jwt, { method: "POST" });
}

export async function deleteSession(
  sessionId: string,
  jwt: string,
): Promise<StatusResponse> {
  return fetchWithAuth<StatusResponse>(`/api/session/${sessionId}`, jwt, {
    method: "DELETE",
  });
}

export async function getMessages(
  sessionId: string,
  jwt: string,
): Promise<Message[]> {
  return fetchWithAuth<Message[]>(`/api/messages/${sessionId}`, jwt);
}

export async function getUser(jwt: string): Promise<UserResponse> {
  return fetchWithAuth<UserResponse>("/api/user/me", jwt);
}

export async function getUsers(jwt: string): Promise<UserResponse[]> {
  return fetchWithAuth<UserResponse[]>("/api/user", jwt);
}

export async function createUser(
  username: string,
  password: string,
  roles: RoleType[],
  jwt: string,
): Promise<StatusResponse> {
  return fetchWithAuth<StatusResponse>("/api/user", jwt, {
    method: "POST",
    body: JSON.stringify({ username, password, roles }),
  });
}

export async function updateUser(
  id: number,
  username: string,
  password: string | undefined,
  roles: RoleType[],
  jwt: string,
): Promise<StatusResponse> {
  const payload = password
    ? { username, password, roles }
    : { username, roles };
  return fetchWithAuth<StatusResponse>(`/api/user/${id}`, jwt, {
    method: "PATCH",
    body: JSON.stringify(payload),
  });
}

export async function deleteUser(
  id: number,
  jwt: string,
): Promise<StatusResponse> {
  return fetchWithAuth<StatusResponse>(`/api/user/${id}`, jwt, {
    method: "DELETE",
  });
}

export async function getToken(formData: FormData): Promise<Token> {
  //https://github.com/microsoft/TypeScript/issues/30584#issuecomment-1865354582
  const data = new URLSearchParams(
    formData as unknown as Record<string, string>,
  );
  const response = await fetch("/api/login", {
    method: "POST",
    body: data,
  });
  if (response.ok) {
    return response.json();
  }
  throw new Error(response.statusText);
}

export async function getQueryLatency(jwt: string): Promise<QueryLatency[]> {
  return fetchWithAuth<QueryLatency[]>("/api/telemetry/latency/query", jwt);
}

export async function getIngestionLatency(
  jwt: string,
): Promise<QueryLatency[]> {
  return fetchWithAuth<QueryLatency[]>("/api/telemetry/latency/ingest", jwt);
}

export async function getQueryTools(jwt: string): Promise<QueryTools[]> {
  return fetchWithAuth<QueryTools[]>("/api/telemetry/tools/query", jwt);
}

export async function getKnowledgeBases(jwt: string): Promise<KnowledgeBase> {
  return fetchWithAuth<KnowledgeBase>("/api/knowledge_base", jwt);
}

export async function uploadFileToKnowledgeBase(
  kbName: string,
  formData: FormData,
  jwt: string,
): Promise<StatusResponse> {
  const response = await fetch(`/api/knowledge_base/${kbName}/ingest`, {
    method: "POST",
    headers: { Authorization: `Bearer ${jwt}` },
    body: formData,
  });
  if (response.ok) {
    return response.json();
  }
  throw new Error(await response.text());
}

export async function invokeAgent(
  selectedAgent: AgentSelections,
  query: string,
  jwt: string,
  sessionId: string,
  onMessage: (message: ChatToken) => void,
): Promise<void> {
  const url = new URL(
    `/ws/${selectedAgent}?${new URLSearchParams({
      session_id: sessionId,
      token: jwt,
    })} `,
    //see vite.config.ts
    import.meta.env.DEV ? "http://localhost:3000" : window.location.href,
  );
  //handles https and wss too since both end in s
  url.protocol = url.protocol.replace("http", "ws");
  const ws = new WebSocket(url);
  ws.onopen = () => {
    ws.send(query);
  };
  ws.onmessage = (event) => {
    onMessage(JSON.parse(event.data));
  };
  await new Promise<void>((res, rej) => {
    ws.onclose = () => {
      res();
    };
    ws.onerror = rej;
  });
}
