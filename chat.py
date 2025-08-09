from llama_index.core.llms import LLM
from llama_index.core.prompts import RichPromptTemplate
from llama_index.core import VectorStoreIndex
from llama_index.core.tools import QueryEngineTool, ToolMetadata


def get_tools(
    llm: LLM,
    vector_store_index: VectorStoreIndex,
    prompt_template: RichPromptTemplate,
) -> list[QueryEngineTool]:
    query_engine = vector_store_index.as_query_engine(
        llm, text_qa_template=prompt_template, streaming=True, similarity_top_k=5
    )
    return [
        QueryEngineTool(
            query_engine=query_engine,
            metadata=ToolMetadata(
                name="vector_index_past_conversation",
                description="useful for when you want to add context from past conversations",
            ),
        )
    ]


def get_response(
    llm: LLM,
    vector_store_index: VectorStoreIndex,
    query: str,
    prompt_template: RichPromptTemplate,  # Callable[[str, str], str],
):
    query_engine = vector_store_index.as_query_engine(
        llm, text_qa_template=prompt_template, streaming=True, similarity_top_k=5
    )
    response = query_engine.query(query)
    # response.print_response_stream()
    for text in response.response_gen:
        print(text)
        # do something with text as they arrive.
        # pass
    # chat_engine = vector_store_index.as_chat_engine()
    # streaming_response = chat_engine.stream_chat("Tell me a joke.")

    # query_engine = vector_store_index.as_query_engine()
    # response = query_engine.query("Who is Paul Graham.")

    # for token in streaming_response.response_gen:
    #    print(token, end="")
