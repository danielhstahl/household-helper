import type {
  QueryLatency,
  QueryTools,
  Token,
  UserResponse,
  Message,
  SessionDB,
  KnowledgeBase,
} from "./models";
export interface StatusResponse {
  status: string;
}
const buildUrl = (base: string, sessionId: string | undefined) => {
  return sessionId
    ? `${base}?` +
        new URLSearchParams({
          session_id: sessionId,
        })
    : base;
};
const getHeaders = (jwt: string) => ({
  "Content-Type": "application/json",
  Authorization: `Bearer ${jwt}`,
});

export const sendHelper = (text: string, jwt: string, sessionId: string) => {
  return fetch(buildUrl("/api/helper", sessionId), {
    method: "POST",
    body: JSON.stringify({ text }),
    headers: getHeaders(jwt),
  });
};

export const sendTutor = (text: string, jwt: string, sessionId: string) => {
  return fetch(buildUrl("/api/tutor", sessionId), {
    method: "POST",
    body: JSON.stringify({ text }),
    headers: getHeaders(jwt),
  });
};

export async function getSessions(jwt: string): Promise<SessionDB[]> {
  const response = await fetch("/api/session", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function getMostRecentSession(jwt: string): Promise<SessionDB> {
  const response = await fetch("/api/session/recent", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function createSession(jwt: string): Promise<SessionDB> {
  const response = await fetch("/api/session", {
    method: "POST",
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function deleteSession(
  sessionId: string,
  jwt: string,
): Promise<StatusResponse> {
  const response = await fetch(`/api/session/${sessionId}`, {
    method: "DELETE",
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function getMessages(
  sessionId: string,
  jwt: string,
): Promise<Message[]> {
  const response = await fetch(`/api/messages/${sessionId}`, {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}
export async function getUser(jwt: string): Promise<UserResponse> {
  const response = await fetch("/api/user/me", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}
export async function getUsers(jwt: string): Promise<UserResponse[]> {
  const response = await fetch("/api/user", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function createUser(
  username: string,
  password: string,
  roles: string[],
  jwt: string,
): Promise<StatusResponse> {
  const response = await fetch("/api/user", {
    method: "POST",
    body: JSON.stringify({ username, password, roles }),
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function updateUser(
  id: number,
  username: string,
  password: string | undefined,
  roles: string[],
  jwt: string,
): Promise<StatusResponse> {
  const payload = password
    ? { username, password, roles }
    : { username, roles };
  const response = await fetch(`/api/user/${id}`, {
    method: "PATCH",
    body: JSON.stringify(payload),
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function deleteUser(
  id: number,
  jwt: string,
): Promise<StatusResponse> {
  const response = await fetch(`/api/user/${id}`, {
    method: "DELETE",
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
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
    const result = await response.json();
    return result;
  }
  throw new Error(response.statusText);
}

export const streamText = (
  onNewText: (_: string) => void,
  onDone: () => void,
) => {
  return async (r: ReadableStreamDefaultReader) => {
    let done = false;
    let value;
    const dec = new TextDecoder();
    while (!done) {
      ({ value, done } = await r.read());
      const strVal = dec.decode(value, { stream: true });
      onNewText(strVal);
    }
    onDone();
  };
};

export async function getQueryLatency(jwt: string): Promise<QueryLatency[]> {
  const response = await fetch("/api/telemetry/latency/query", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function getIngestionLatency(
  jwt: string,
): Promise<QueryLatency[]> {
  const response = await fetch("/api/telemetry/latency/ingest", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function getQueryTools(jwt: string): Promise<QueryTools[]> {
  const response = await fetch("/api/telemetry/tools/query", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function getKnowledgeBases(jwt: string): Promise<KnowledgeBase> {
  const response = await fetch("/api/knowledge_base", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}

export async function uploadFileToKnowledgeBase(
  kbId: number,
  formData: FormData,
  jwt: string,
): Promise<StatusResponse> {
  const response = await fetch(`/api/knowledge_base/${kbId}/ingest`, {
    method: "POST",
    headers: { Authorization: `Bearer ${jwt}` },
    body: formData,
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
}
