from llama_index.core.llms import LLM
from llama_index.core.prompts import RichPromptTemplate
from llama_index.core import VectorStoreIndex
from llama_index.core.tools import QueryEngineTool, ToolMetadata
from llama_index.core.memory import VectorMemoryBlock
from llama_index.core.vector_stores.types import BasePydanticVectorStore
from llama_index.core.base.embeddings.base import BaseEmbedding
from llama_index.core.memory import Memory
from llama_index.core.memory.memory import BaseMemoryBlock, InsertMethod


## Not used at the moment
# originally used for storing old chats, but
# this is taken care of by Memory
def get_tools(
    llm: LLM,
    vector_store_index: VectorStoreIndex,
    prompt_template: RichPromptTemplate,
    past_conversation_tool: str,
) -> list[QueryEngineTool]:
    query_engine = vector_store_index.as_query_engine(
        llm,
        text_qa_template=prompt_template,
        streaming=True,
        similarity_top_k=5,
        use_async=True,
    )
    return [
        QueryEngineTool(
            query_engine=query_engine,
            metadata=ToolMetadata(
                name=past_conversation_tool,
                description="useful for when you want to add context from past conversations",
            ),
        )
    ]


def get_memory(
    embedding_model: BaseEmbedding,
    vector_store: BasePydanticVectorStore,
    session_id: str,
) -> Memory:
    blocks: list[BaseMemoryBlock] = [
        VectorMemoryBlock(
            name="vector_memory",
            # required: pass in a vector store like qdrant, chroma, weaviate, milvus, etc.
            vector_store=vector_store,
            priority=0,
            embed_model=embedding_model,
            similarity_top_k=5,
        )
    ]
    # does this store context/messages in the default table
    # (llama_index_memory) or in the table from the vector_store?
    return Memory.from_defaults(
        session_id=session_id, memory_blocks=blocks, insert_method=InsertMethod.SYSTEM
    )
