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

export const getSessions = async (jwt: string) => {
  const response = await fetch("/api/session", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const getMostRecentSession = async (jwt: string) => {
  const response = await fetch("/api/session/recent", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const createSession = async (jwt: string) => {
  const response = await fetch("/api/session", {
    method: "POST",
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const deleteSession = async (sessionId: string, jwt: string) => {
  const response = await fetch(`/api/session/${sessionId}`, {
    method: "DELETE",
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const getMessages = async (sessionId: string, jwt: string) => {
  const response = await fetch(`/api/messages/${sessionId}`, {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};
export const getUser = async (jwt: string) => {
  const response = await fetch("/api/user/me", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};
export const getUsers = async (jwt: string) => {
  const response = await fetch("/api/user", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const createUser = async (
  username: string,
  password: string,
  roles: string[],
  jwt: string,
) => {
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
};

export const updateUser = async (
  id: number,
  username: string,
  password: string | undefined,
  roles: string[],
  jwt: string,
) => {
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
};

export const deleteUser = async (id: number, jwt: string) => {
  const response = await fetch(`/api/user/${id}`, {
    method: "DELETE",
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const getToken = async (formData: FormData) => {
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
};

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

export const getQueryLatency = async (jwt: string) => {
  const response = await fetch("/api/telemetry/latency/query", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const getIngestionLatency = async (jwt: string) => {
  const response = await fetch("/api/telemetry/latency/ingest", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};

export const getQueryTools = async (jwt: string) => {
  const response = await fetch("/api/telemetry/tools/query", {
    headers: getHeaders(jwt),
  });
  if (response.ok) {
    const result = await response.json();
    return result;
  }
  throw new Error(await response.text());
};
