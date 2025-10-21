const commonLanguages = [
  "javascript",
  "typescript",
  "python",
  "rust",
  "html",
  "css",
  "csharp",
  "sql",
  "go",
  "yaml",
];
// llms sometimes mess up syntax.
// parsing this text ensures that code is actually on separate lines
export const parseText = (text: string) => {
  return commonLanguages
    .reduce((aggr, curr) => {
      return aggr.replaceAll("```" + curr, "```" + curr + "\n");
    }, text)
    .replaceAll("```", "\n```");
};
