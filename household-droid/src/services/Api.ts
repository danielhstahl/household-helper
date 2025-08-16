export const sendQuery = (text: string) => {
  return (
    fetch("/query", {
      method: "POST",
      body: JSON.stringify({ text }),
      headers: { "Content-Type": "application/json" },
    })
      // Retrieve its body as ReadableStream
      .then((response) => {
        return response.body!.getReader();
      })
  );
};

export const sendTutor = (text: string) => {
  return (
    fetch("/tutor", {
      method: "POST",
      body: JSON.stringify({ text }),
      headers: { "Content-Type": "application/json" },
    })
      // Retrieve its body as ReadableStream
      .then((response) => {
        return response.body!.getReader();
      })
  );
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
