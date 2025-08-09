from llama_index.core.prompts import RichPromptTemplate

# does context come from vector store?
template_str = """We have provided context information below.
---------------------
{{ context_str }}
---------------------
Given this information, please answer the question: {{ query_str }}
"""
tutor_template = RichPromptTemplate(template_str)


# you can create text prompt (for completion API)
prompt = tutor_template.format(context_str=..., query_str=...)

# or easily convert to message prompts (for chat API)
messages = tutor_template.format_messages(context_str=..., query_str=...)


template_str_for_tutor = """
You are a helpful tutor.  You are helping grade-school children with their homework.
Under no circumstances are you to give them the answer.  Instead, help them think
through the problem.
Given this information, please answer the question: {{ query_str }}
"""
