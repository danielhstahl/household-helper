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

export const sendQuery = (text: string, jwt: string, sessionId: string) => {
  return fetch(buildUrl("/api/query", sessionId), {
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

export const getSessions = (jwt: string) => {
  return fetch("/api/session", {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const getMostRecentSession = (jwt: string) => {
  return fetch("/api/session/recent", {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const createSession = (jwt: string) => {
  return fetch("/api/session", {
    method: "POST",
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const deleteSession = (sessionId: string, jwt: string) => {
  return fetch(`/api/session/${sessionId}`, {
    method: "DELETE",
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const getMessages = (sessionId: string, jwt: string) => {
  return fetch(`/api/messages/${sessionId}`, {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};
export const getUser = async (jwt: string) => {
  const response = await fetch("/api/user/me", {
    headers: getHeaders(jwt),
  });
  const result = await response.json();
  if (response.ok) {
    return result;
  }
  throw new Error(result.detail);
};
export const getUsers = async (jwt: string) => {
  const response = await fetch("/api/user", {
    headers: getHeaders(jwt),
  });
  const result = await response.json();
  if (response.ok) {
    return result;
  }
  throw new Error(result.detail);
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
  const result = await response.json();
  if (response.ok) {
    return result;
  }
  throw new Error(result.detail);
};

export const updateUser = (
  id: number,
  username: string,
  password: string | undefined,
  roles: string[],
  jwt: string,
) => {
  const payload = password
    ? { username, password, roles }
    : { username, roles };
  return fetch(`/api/user/${id}`, {
    method: "PATCH",
    body: JSON.stringify(payload),
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const deleteUser = (id: number, jwt: string) => {
  return fetch(`/api/user/${id}`, {
    method: "DELETE",
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
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
  const result = await response.json();
  if (response.ok) {
    return result;
  }
  throw new Error(result.detail);
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
