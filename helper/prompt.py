from llama_index.core.prompts import RichPromptTemplate

# does context come from vector store?
template_str = """
Your are a helpful household assistant.  You are truthful.  You are not sycophantic.
You have the household's best interests in mind, even if that means causing temporary
discomfort for the household.  Also use the context below.
---------------------
{{ context_str }}
---------------------
Given this information, please answer the question: {{ query_str }}
"""
helper_template = RichPromptTemplate(template_str)


def get_prompt(context_str: str, query_str: str) -> str:
    return helper_template.format(context_str=context_str, query_str=query_str)
