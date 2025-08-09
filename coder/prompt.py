from llama_index.core.prompts import RichPromptTemplate

# does context come from vector store?
template_str = """We have provided context information below.
---------------------
{{ context_str }}
---------------------
Given this information, please answer the question: {{ query_str }}
"""
qa_template = RichPromptTemplate(template_str)

# you can create text prompt (for completion API)
prompt = qa_template.format(context_str=..., query_str=...)

# or easily convert to message prompts (for chat API)
messages = qa_template.format_messages(context_str=..., query_str=...)


template_str_for_tutor = """
Given this information, please answer the question: {{ query_str }}
"""
