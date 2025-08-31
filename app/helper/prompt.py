from llama_index.core.prompts import RichPromptTemplate

# does context come from vector store? (yes)
template_str = """
Also use the context below.
---------------------
{{ context_str }}
---------------------
Given this context, response to this query: {{ query_str }}
"""
helper_template = RichPromptTemplate(template_str)


def get_prompt(context_str: str, query_str: str) -> str:
    return helper_template.format(context_str=context_str, query_str=query_str)
